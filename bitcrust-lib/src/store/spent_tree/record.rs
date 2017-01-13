
use std::mem;

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;

use store::spent_tree::SpendingError;

/// A record is a 16 byte structure that points to either a
/// * blockheader
/// * transaction
/// * transaction-output
///
/// The skips point to other Records; at least the previous.
///
/// The exact format is still in work-in-progress.
///
#[derive(Debug,Copy,Clone)]
pub struct Record {
    pub ptr:   FilePtr,
    pub skips: [i16;4]
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
            rec.skips = [0;4];
            return;
        }
        let previous = previous.unwrap();

        assert!(self.ptr.file_pos() != previous.ptr.file_pos());

        rec.skips = unsafe { mem::transmute(previous.ptr) };//.to_u64();
    }



    pub fn seek_and_set(self, fileset: &mut FlatFileSet) -> Result<isize, SpendingError> {

        let mut count = 0;

        const LARGE_DIFF: i64 = 10000;

        // seek_rec is the one we seek (self)
        let seek_rec: &mut Record = fileset.read_fixed(self.ptr);


        // this is the transaction position we seek (the filepos stripped of the output-index metadata)
        let seek_filenr_pos = seek_rec.ptr.filenumber_and_pos();
        let seek_plus = seek_filenr_pos + LARGE_DIFF;

        // these are the pointers that will be stored in rec. By default, they just point to the
        // previous
        seek_rec.skips = [-1; 4];

        let mut cur = self.prev_in_block();

        if seek_rec.ptr.is_transaction() {
            return Ok(0);
        }

        debug_assert!(seek_rec.ptr.is_output()); // these are the only ones to search for

        loop
        {
            // cur_rec is the one we are comparing
            let cur_rec: &Record = fileset.read_fixed(cur.ptr);

            let cur_filenr_pos = cur_rec.ptr.filenumber_and_pos();

            println!("Seeking {:?} @ {:?} = {:?}", seek_rec, cur, cur_rec);



            if cur_rec.skips == [0;4] {

                return Err(SpendingError::OutputNotFound);

            } else if cur_rec.ptr.is_blockheader() || cur_rec.ptr.is_guard_blockheader() {

                cur = cur.prev(fileset);
                continue;
            }

            let skipper = if cur_filenr_pos == seek_filenr_pos {

                if cur_rec.ptr.is_transaction() {

                    // we've found the transaction of the output before we
                    // found the output. So we're all good
                    return Ok(count)
                }
                else if cur_rec.ptr.output_index() == seek_rec.ptr.output_index() {
                    return Err(SpendingError::OutputAlreadySpent);
                }

                // the first skip skips over everything that is from the same tx
                0


            } else if  seek_plus < cur_filenr_pos {

                1

            } else if seek_filenr_pos < cur_filenr_pos {

                2
            } else {

                3
            };

            cur = cur.offset(cur_rec.skips[skipper]);

        }

    }


    /// Get the previous pointer; this mirrors the ^^ set_previous function
    pub fn prev(self, fileset: &mut FlatFileSet) -> RecordPtr {
        let  rec: &mut Record = fileset.read_fixed(self.ptr);

        if !rec.ptr.is_guard_blockheader() {
            self.prev_in_block()
        }
        else {
            let skips:u64 = unsafe { mem::transmute(rec.skips) };
            RecordPtr::new(FilePtr::from_u64(skips))
        }
    }

    pub fn offset(self, offset: i16) -> RecordPtr {
        RecordPtr::new(self.ptr.offset(offset as i32 * 16 ))
    }

    pub fn prev_in_block(self) -> RecordPtr {
        RecordPtr::new(self.ptr.offset(-16))
    }

    pub fn next_in_block(self) -> RecordPtr {
        RecordPtr::new(self.ptr.offset(16))
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
            skips: [0;4]
        }
    }




}



#[cfg(test)]
mod tests {





}