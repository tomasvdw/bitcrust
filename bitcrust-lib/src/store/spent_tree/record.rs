

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;


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
    skips: u64
}



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
            rec.set_bits(0,3);
            return;
        }
        let previous = previous.unwrap();

        assert!(self.ptr.file_pos() != previous.ptr.file_pos());

        // different file or wrong direction -> we'll store a full pointer
        if self.ptr.file_pos() < previous.ptr.file_pos() ||
            self.ptr.file_number() != previous.ptr.file_number()
        {
            rec.skips = previous.ptr.to_u64();
            return;
        }

        let diff = (self.ptr.file_pos() - previous.ptr.file_pos()) as u64;
        match diff {
            1 => {
                rec.set_bits(0,1); //.skips[0] = 0b0100_0000;
            },
            2 ... 0b0011_1111_1111_1111 => {
                rec.set_bits(0,2);
                rec.set_bits(2, diff);
            },
            _ => {
                rec.skips = previous.ptr.to_u64();
                return;
            }
        }

    }


    /// Get the previous pointer; this mirrors the ^^ set_previous function
    pub fn prev(self, fileset: &mut FlatFileSet) -> RecordPtr {
        let  rec: &mut Record = fileset.read_fixed(self.ptr);

        if !rec.ptr.is_blockheader() {
            self.prev_in_block()
        }
        else {

            match rec.get_bits(0,2) {

                1 => RecordPtr::new(self.ptr.offset(-16)),

                2 => RecordPtr::new(self.ptr.offset(- (rec.get_bits(2,62) as i32))),

                _ => RecordPtr::new(FilePtr::from_u64(rec.skips)),

            }

        }
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

            if result.skips == 0 {
                self.cur_ptr.ptr = FilePtr::null()
            };
            Some(result)
        }
    }
}


impl Record {

    pub fn get_bits(&self, start: u64, length: u64) -> u64 {

        (self.skips >> start) & ((1<<length)-1)
    }

    pub fn set_bits(&mut self, start: u64, value: u64) {
        self.skips |= value << start
    }

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
            skips: 0
        }
    }

    pub fn set_skip_previous(&mut self) {
        self.set_bits(0,1)
    }



}



#[cfg(test)]
mod tests {





}