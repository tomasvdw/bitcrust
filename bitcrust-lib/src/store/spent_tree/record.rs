
use std::mem;

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;

use store::spent_tree::SpendingError;

use super::SpentTreeStats;

/// A record is a 16 byte structure that points to either a
/// * blockheader
/// * transaction
/// * transaction-output
///
/// The skips point to other Records; at least the previous.
///
/// The exact format is still in work-in-progress.
///
const SKIP_FIELDS: usize = 12;

#[derive(Debug,Copy,Clone)]
pub struct Record {
    pub ptr:   FilePtr,
    pub skips: [i16;SKIP_FIELDS]
}




enum CompareResult {
    TransactionFound,
    OutputFound,
    NoneFound,
    CurrentIsLarger,
    CurrentIsSmaller
}
/*
fn compare_fileptrs(current: FilePtr, search: FilePtr ) -> CompareResult {
    if current.filenumber_and_pos() == search.filenumber_and_pos() {

        if current.is_transaction() {

            return TransactionFound;
        }
        else {

        }
    }

}
*/
/// A filepointer that points to a record in the SpentTree
#[derive(Debug,Copy,Clone)]
pub struct RecordPtr {
    pub ptr: FilePtr
}

impl RecordPtr {

    pub fn new(ptr: FilePtr) -> Self {
        RecordPtr { ptr: ptr }
    }

    pub fn set_previous(self, fileset: &mut FlatFileSet, previous: Option<RecordPtr>) {

        let  rec: &mut Record = fileset.read_fixed(self.ptr);

        if previous.is_none() {
            rec.skips = [0;SKIP_FIELDS];
            return;
        }
        let previous = previous.unwrap();

        assert!(self.ptr.file_pos() != previous.ptr.file_pos());

        rec.set_ptr_in_skips(previous);//.to_u64();
    }



    pub fn seek_and_set(self, stats: &mut SpentTreeStats, fileset: &mut FlatFileSet) -> Result<isize, SpendingError> {

        stats.inputs += 1;

        let mut count = 0;

        const DELTA: [i64; SKIP_FIELDS] = [
            - 256 * 256,
            - 16 * 256,
            -4 * 256 ,
            0,
            4 * 256,
            16 * 256,
            64 * 256 ,
            256 * 256  ,
            16 * 256 * 256,
            64 * 256 * 256,
            256 * 256 * 256,
            16 * 256 * 256 * 256 ];

        // seek_rec is the one we seek (self)
        let seek_rec: &mut Record = fileset.read_fixed(self.ptr);


        // this is the transaction position we seek (the filepos stripped of the output-index metadata)
        let seek_filenr_pos = seek_rec.ptr.filenumber_and_pos();

        // all filenr_pos we've passed are at least this high
        let mut minimal_filenr_pos = seek_filenr_pos;

        // these are the pointers that will be stored in rec. By default, they just point to the
        // previous
        seek_rec.skips = [-1; SKIP_FIELDS];

        // for lack of a better name, `skip_r` traces which of the four skips of seek_rec are still
        // "following" the jumps. Initially all are (which will cause seek_rec.skips to be all set
        // to -1 again), and once seek_plus[0] is cur_filenr_pos is less then
        let mut skip_r = 0;

        let seek_plus: Vec<i64>  = DELTA.iter().map(|n| seek_filenr_pos + n).collect();
        let seek_minus: Vec<i64> = DELTA.iter().map(|n| seek_filenr_pos - n).collect();


        let mut cur = self.prev_in_block();

        let is_tx =  seek_rec.ptr.is_transaction();

        //if is_tx {
        //    return Ok(0);
        //}

        let mut cur_rec: &Record = fileset.read_fixed(cur.ptr);

        loop {

            stats.seeks += 1;

            count += 1;

            let cur_filenr_pos = cur_rec.ptr.filenumber_and_pos();

            //println!("Seeking {:?} @ {:?} = {:?}", seek_rec, cur, cur_rec);

            if cur_rec.skips == [0; SKIP_FIELDS] {
                if is_tx {
                    return Ok(count);
                }
                else {
                    // for now panic is easier for traces
                    //panic!(format!("Output {:?} not found at {:?}", seek_rec, cur.ptr));
                    return Err(SpendingError::OutputNotFound);
                }

            } else if cur_rec.ptr.is_blockheader() || cur_rec.ptr.is_guard_blockheader() {

                stats.jumps += 1;
                cur = cur.prev(fileset);
                cur_rec = fileset.read_fixed(cur.ptr);
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

            if cur.ptr.file_number() == self.ptr.file_number() {

                let diff: i64 = (cur.ptr.file_pos() as i64 - self.ptr.file_pos() as i64) /
                    mem::size_of::<Record>() as i64;

                if diff > i16::min_value() as i64 && diff < i16::max_value() as i64 {
                    for n in skip_r..DELTA.len() {
                        seek_rec.skips[n] = diff as i16;
                    }
                }
            }
            else {
                panic!("Multiple files still untested");
            }


            if minimal_filenr_pos > cur_filenr_pos {
                minimal_filenr_pos = cur_filenr_pos;
            }


            let mut skip = -1;
            for n in (0..SKIP_FIELDS).rev() {
                if seek_plus[n] < cur_filenr_pos {

                    stats.total_move += cur_rec.skips[n] as i64;
                    stats.use_diff[n] += 1;
                    skip = cur_rec.skips[n];

                    if minimal_filenr_pos > cur_filenr_pos - DELTA[n] {
                        minimal_filenr_pos = cur_filenr_pos - DELTA[n];
                    }
                    break;
                }
            }

            while skip_r < SKIP_FIELDS && seek_minus[skip_r] >= minimal_filenr_pos {
                skip_r += 1;
                if is_tx && skip_r > 0 {
                    return Ok(count);
                }
            }

            cur = cur.offset(skip);
            /*let cur_ptr = cur_rec as *const Record;
            let nxt_ptr = unsafe { cur_ptr.offset(skip as isize * 16) };
            cur_rec = unsafe { nxt_ptr.as_ref().unwrap() };
            */
            cur_rec = fileset.read_fixed(cur.ptr);

        }

    }

    pub fn seek_and_set_seqscan(self, fileset: &mut FlatFileSet) -> Result<isize, SpendingError> {

        let mut count = 0;

        // seek_rec is the one we seek (self)
        let seek_rec: &mut Record = fileset.read_fixed(self.ptr);
        seek_rec.skips = [-1; SKIP_FIELDS];

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



            if cur_rec.skips == [0;SKIP_FIELDS] {

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

    pub fn offset(self, offset: i16) -> RecordPtr {
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


    pub fn iter(self, fileset: &mut FlatFileSet) -> RecordBackwardsIterator {

        RecordBackwardsIterator {
            cur_ptr:   self,
            fileset:   fileset
        }

    }


}

pub struct RecordBackwardsIterator<'a> {
    cur_ptr:    RecordPtr,
    fileset:    &'a mut FlatFileSet
}


/// Browsing backwards over the entire tree
impl<'a> Iterator for RecordBackwardsIterator<'a> {
    type Item = Record;

    fn next(&mut self) -> Option<Record> {
        if self.cur_ptr.ptr.is_null()  {
            None
        }
        else {
            self.cur_ptr = self.cur_ptr.prev(self.fileset);
            let result = *self.fileset.read_fixed::<Record>(self.cur_ptr.ptr);

            if result.skips[0] == 0 {
                self.cur_ptr.ptr = FilePtr::null()
            };
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
            skips: [0;SKIP_FIELDS]
        }
    }


    fn set_ptr_in_skips(&mut self, ptr: RecordPtr) {

        let cv: [u64;3] = [ptr.ptr.to_u64(),0,0];
        self.skips = unsafe { mem::transmute(cv) };

    }

    fn get_ptr_from_skips(&mut self) -> RecordPtr {
        let cv: [u64;3] = unsafe { mem::transmute(self.skips) };

        RecordPtr::new(FilePtr::from_u64(cv[0]))
    }


}



#[cfg(test)]
mod tests {



    fn test_spenttree_large() {

    }


}