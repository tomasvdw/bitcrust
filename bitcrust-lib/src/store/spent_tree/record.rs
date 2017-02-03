
#[macro_use]
use slog;

use std::mem;
use std::fmt;

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;

use store::spent_tree::SpendingError;

use super::SpentTreeStats;

use super::params;

/// A record is a 16 byte structure that points to either a
/// * blockheader
/// * transaction
/// * transaction-output
///
/// The skips point to other Records; at least the previous.
///
/// The exact format is still in work-in-progress.
///

#[derive(Copy,Clone)]
pub struct Record {
    pub ptr:   FilePtr,
    pub skips: [i16;params::SKIP_FIELDS]
}




/// A filepointer that points to a record in the SpentTree
#[derive(Copy,Clone)]
pub struct RecordPtr {
    pub ptr: FilePtr
}


impl fmt::Debug for RecordPtr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        write!(fmt, "{:?}", self.ptr)

    }
}

impl fmt::Debug for Record {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        write!(fmt, "[ptr={:?} skip={:04x}|{:04x}|{:04x}|{:04x}]",
               self.ptr, self.skips[0],self.skips[1],self.skips[2],self.skips[3])

        /*write!(fmt, "[ptr={:?} skip={:04x}|{:04x}|{:04x}|{:04x}|{:04x}|{:04x}|{:04x}|{:04x}|{:04x}|{:04x}|{:04x}|{:04x}]",
               self.ptr, self.skips[0],self.skips[1],self.skips[2],self.skips[3],self.skips[4],self.skips[5],self.skips[6],
               self.skips[7],self.skips[8],self.skips[9],self.skips[10],self.skips[11])
*/
    }
}


impl RecordPtr {

    pub fn new(ptr: FilePtr) -> Self {
        RecordPtr { ptr: ptr }
    }

    pub fn set_previous(self, fileset: &mut FlatFileSet, previous: Option<RecordPtr>) {

        let  rec: &mut Record = fileset.read_fixed(self.ptr);

        if previous.is_none() {
            rec.skips = [0;params::SKIP_FIELDS];
            return;
        }
        let previous = previous.unwrap();

        assert!(self.ptr.file_pos() != previous.ptr.file_pos());

        rec.set_ptr_in_skips(previous);//.to_u64();
    }




    pub fn to_index(self) -> usize {
        if self.ptr.file_number() != 0 {
            panic!("Only single file supported")
        }
        else {
            (self.ptr.file_pos() -16) / mem::size_of::<Record>()
        }

    }


    pub fn seek_and_set(&mut self,
                        seek_rec: &mut Record,
                        seek_idx: usize,
                        records:  &[Record],
                        logger:   &slog::Logger,
                        stats:    &mut SpentTreeStats) -> Result<i64, SpendingError> {

        let mut count = 0;
        stats.inputs += 1;

        trace!(logger, "Records loaded");

        let mut cur_idx = seek_idx - 1;        // cur  will walk through the skip-list

        trace!(logger, format!("FL# Start search for {:?} {:?} ", seek_rec, seek_idx));

        // this is the transaction position we seek (the filepos stripped of the output-index metadata)
        let seek_filenr_pos = seek_rec.ptr.filenumber_and_pos();

        // all filenr_pos we've passed are at least this high
        let mut minimal_filenr_pos = seek_filenr_pos;

        // these are the pointers that will be stored in rec. By default, they just point to the
        // previous
        seek_rec.skips = [-1; params::SKIP_FIELDS];


        // for lack of a better name, `skip_r` traces which of the four skips of seek_rec are still
        // "following" the jumps. Initially all are (which will cause seek_rec.skips to be all set
        // to -1 again), and once seek_plus[0] is cur_filenr_pos is too small, skip_r will increase
        let mut skip_r = 0;

        let seek_plus: Vec<i64>  = params::DELTA.iter().map(|n| seek_filenr_pos + n).collect();
        let seek_minus: Vec<i64> = params::DELTA.iter().map(|n| seek_filenr_pos - n).collect();


        let is_tx =  seek_rec.ptr.is_transaction();

        //if is_tx { return Ok(0); }

        loop {
            let cur_rec: Record = records[cur_idx];

            stats.seeks += 1;
            count += 1;

            let cur_filenr_pos = cur_rec.ptr.filenumber_and_pos();

            if cur_rec.skips == [0; params::SKIP_FIELDS] {
                // we're at the end

                if is_tx {
                    // a transaction is supposed to be not found
                    trace!(logger, format!("FL# Done after {:?} seeks", stats.seeks));
                    return Ok(count);
                }
                else {
                    //panic!(format!("Output {:?} not found at {:?}", seek_rec, cur.ptr));
                    return Err(SpendingError::OutputNotFound);
                }

            } else if cur_rec.ptr.is_blockheader() || cur_rec.ptr.is_guard_blockheader() {

                // blockheaders don't have skips; interpret as a full fileptr
                trace!(logger, format!("FL# {:?}-{:?} JUMPING", cur_idx, cur_rec));

                stats.jumps += 1;

                cur_idx = cur_rec.get_ptr_from_skips().to_index();
                continue;
            }

            if seek_filenr_pos == cur_filenr_pos {
                if is_tx || cur_rec.ptr.is_transaction() {
                    // we've found the transaction of the output before we
                    // found the same output. So we're all good

                    return Ok(count)

                } else if cur_rec.ptr.output_index() == seek_rec.ptr.output_index() {

                    //panic!(format!("Output {:?} double spent {:?}", seek_rec, cur.ptr));
                    return Err(SpendingError::OutputAlreadySpent);
                }
            }


            // See if there are skip-values we need to update in the record were seeking
            if skip_r < params::DELTA.len() {

                let diff: i64 = cur_idx as i64 - seek_idx as i64;
                if diff > i16::min_value() as i64 && diff < i16::max_value() as i64 {
                    for n in skip_r..params::DELTA.len() {
                        seek_rec.skips[n] = diff as i16;
                    }
                }
                /*else {
                    println!("Proc Diff too large {:?}", diff);
                }*/

                if minimal_filenr_pos > cur_filenr_pos {
                    minimal_filenr_pos = cur_filenr_pos;
                }


                while skip_r < params::SKIP_FIELDS && seek_minus[skip_r] >= minimal_filenr_pos {
                    skip_r += 1;


                    if is_tx && skip_r >= params::TX_NEEDED_SKIPS {
                        return Ok(stats.seeks);
                    }
                }
            }

            let mut skip = -1;
            for n in (0..params::SKIP_FIELDS).rev() {
                if seek_plus[n] < cur_filenr_pos {
                    stats.total_move += cur_rec.skips[n] as i64;
                    stats.use_diff[n] += 1;
                    skip = cur_rec.skips[n];

                    if minimal_filenr_pos > cur_filenr_pos - params::DELTA[n] {
                        minimal_filenr_pos = cur_filenr_pos - params::DELTA[n];
                    }
                    break;
                }
            }


            /*if full_dump {
                let mut cur = RecordPtr::new(FilePtr::new(0, (16 + cur_idx * mem::size_of::<Record>()) as u32));
                let target = cur.offset(skip);
                loop {
                    let c: &Record = fileset.read_fixed(cur.ptr);
                    println!("FL# {:?}-{:?} <seek {:?}={:?}>", cur, c, skip, target);
                    cur = cur.prev(fileset);
                    let c: &Record = fileset.read_fixed(cur.ptr);

                    if cur.ptr == target.ptr {
                        println!("FL# {:?}-{:?} FOUND", cur, c);
                        break;
                    }
                }

            }*/

            cur_idx = (cur_idx as i64 + skip as i64) as usize;




        }

    }

    pub fn seek_and_set_seqscan(self, fileset: &mut FlatFileSet) -> Result<isize, SpendingError> {

        let mut count = 0;

        // seek_rec is the one we seek (self)
        let seek_rec: &mut Record = fileset.read_fixed(self.ptr);
        seek_rec.skips = [-1; params::SKIP_FIELDS];

        // this is the transaction position we seek (the fileptr minus the output-index metadata)
        let seek_filenr_pos = seek_rec.ptr.filenumber_and_pos();

        let mut cur = self.prev_in_block();

        if seek_rec.ptr.is_transaction() {
            return Ok(count);
        }

        debug_assert!(seek_rec.ptr.is_output()); // these are the only ones to search for

        loop
        {
            count += 1;

            // cur_rec is the one we are comparing
            let cur_rec: &Record = fileset.read_fixed(cur.ptr);

            let cur_filenr_pos = cur_rec.ptr.filenumber_and_pos();

            println!("Scanning {:?} @ {:?} = {:?}", seek_rec, cur, cur_rec);



            if cur_rec.skips == [0;params::SKIP_FIELDS] {

                return Err(SpendingError::OutputNotFound);

            } else if cur_rec.ptr.is_blockheader() || cur_rec.ptr.is_guard_blockheader() {

                cur = cur.prev(fileset);
                continue;
            }

            if cur_filenr_pos == seek_filenr_pos {

                if cur_rec.ptr.is_transaction() {

                    // we've found the transaction of the output before we
                    // found the output. So we're all good
                    return Ok(count)
                }
                else if cur_rec.ptr.output_index() == seek_rec.ptr.output_index() {
                    return Err(SpendingError::OutputAlreadySpent);
                }

            };

            cur = cur.offset(-1);

        }

    }


    /// Get the previous pointer; this mirrors the ^^ set_previous function
    pub fn prev(self, fileset: &mut FlatFileSet) -> RecordPtr {
        let  rec: &mut Record = fileset.read_fixed(self.ptr);

        if !rec.ptr.is_guard_blockheader() {
            self.prev_in_block()
        }
        else {
            rec.get_ptr_from_skips()
        }
    }

    pub fn offset(self, offset: i32) -> RecordPtr {
        RecordPtr::new(self.ptr.offset(offset as i32 * mem::size_of::<Record>() as i32  ))
    }

    pub fn prev_in_block(self) -> RecordPtr {
        self.offset(-1)
    }

    pub fn next_in_block(self) -> RecordPtr {
        self.offset(1)
    }

    pub fn get_content_ptr(self, fileset: &mut FlatFileSet) -> FilePtr {
        fileset.read_fixed::<Record>(self.ptr).ptr
    }

    pub fn set_content_ptr(self, fileset: &mut FlatFileSet, new_ptr: FilePtr) {
        let p: &mut FilePtr = &mut fileset.read_fixed::<Record>(self.ptr).ptr;
        let _ = p.atomic_replace(FilePtr::null(), new_ptr);
    }


    pub fn iter(self, fileset: &mut FlatFileSet) -> RecordBlockIterator {

        RecordBlockIterator {
            cur_ptr:   self.next_in_block(),
            fileset:   fileset
        }

    }


}

pub struct RecordBlockIterator<'a> {
    cur_ptr:    RecordPtr,
    fileset:    &'a mut FlatFileSet
}


/// Browsing backwards over the entire tree
impl<'a> Iterator for RecordBlockIterator<'a> {
    type Item = RecordPtr;

    fn next(&mut self) -> Option<RecordPtr> {
        let rec: Record = * self.fileset.read_fixed::<Record>(self.cur_ptr.ptr);

        if rec.ptr.is_blockheader() {
            None
        }
        else {
            let result = self.cur_ptr;
            self.cur_ptr = self.cur_ptr.next_in_block();

            Some(result)
        }
    }
}


impl Record {



    /*
    pub fn previous(&self, fileset: &mut FlatFileSet) -> Option<&Record> {
        match self.bits(0,2) {
            0 => Some( fileset.read_fixed(self.skips_as_fileptr()) ),
            1 => Some( self.before_in_memory(1) ),
            2 => Some( self.before_in_memory( self.skips_bit_3_to_16())),
            3 => None,
            _ => unreachable!()
        }
    }
    */

    pub fn seek(&self) -> Option<&Record> {
        None
    }

    /// This is a preliminary new. To set the proper skip pointers
    /// we must now where we are in the file so we do this aferwards in set_skips
    pub fn new(content: FilePtr) -> Self {
        Record {
            ptr: content,
            skips: [0;params::SKIP_FIELDS]
        }
    }


    fn set_ptr_in_skips(&mut self, ptr: RecordPtr) {

        let cv: [u64;1] = [ptr.ptr.to_u64()];
        self.skips = unsafe { mem::transmute(cv) };

    }

    fn get_ptr_from_skips(&self) -> RecordPtr {
        let cv: [u64;1] = unsafe { mem::transmute(self.skips) };

        RecordPtr::new(FilePtr::from_u64(cv[0]))
    }

}



#[cfg(test)]
mod tests {



    fn test_spenttree_large() {

    }


}