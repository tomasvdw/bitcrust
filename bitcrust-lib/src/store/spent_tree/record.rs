

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
pub struct Record {
    ptr:   FilePtr,
    skips: u64
}




impl Record {

    fn get_bits(&self, start: u64, length: u64) -> u64 {
        (self.skips >> start) & ((2^length)-1)
    }

    fn set_bits(&mut self, start: u64, value: u64) {
        self.skips |= value << start
    }

    /*
    pub fn previous(&self, fileset: &mut FlatFileSet) -> Option<&Record> {
        match self.bits(0,2) {
            0 => Some( fileset.read_fixed(self.skips_as_fileptr()) ),
            1 => Some( self.before_in_memory(1) ),
            2 => Some( self.before_in_memory( self.skips_bit_3_to_16())),
            3 => None,
            _ => panic!()
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



    /// Initiazes the previous pointer and the skip-list for this record
    pub fn set_skips(&mut self, self_ptr: FilePtr, previous: Option<FilePtr>) {
        if previous.is_none() {
            self.set_bits(0,3);
            return;
        }
        let previous = previous.unwrap();

        assert!(self_ptr.file_pos() > previous.file_pos());

        if self_ptr.file_number() != previous.file_number() {
            self.skips = previous.as_u64();
            return;
        }

        let diff = (self_ptr.file_pos() - previous.file_pos()) as u64;
        match diff {
            1 => {
                self.set_bits(0,1); //.skips[0] = 0b0100_0000;
            },
            2 ... 0b0011_1111_1111_1111 => {
                self.set_bits(0,2);
                self.set_bits(2, diff);
            },
            _ => {
                self.skips = previous.as_u64();
                return;
            }
        }

    }
}



#[cfg(test)]
mod tests {



}