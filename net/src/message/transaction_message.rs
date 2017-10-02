use std::io;

use Encode;
use VarInt;
use transactions::*;

///
/// Field Size	Description	Data type	Comments
/// 4	version	int32_t	Transaction data format version (note, this is signed)
/// 0 or 2	flag	optional uint8_t[2]	If present, always 0001, and indicates the presence of witness data
/// 1+	tx_in count	var_int	Number of Transaction inputs (never zero)
/// 41+	tx_in	tx_in[]	A list of 1 or more transaction inputs or sources for coins
/// 1+	tx_out count	var_int	Number of Transaction outputs
/// 9+	tx_out	tx_out[]	A list of 1 or more transaction outputs or destinations for coins
/// 0+	tx_witnesses	tx_witness[]	A list of witnesses, one for each input; omitted if flag is omitted above
/// 4	lock_time	uint32_t	The block number or timestamp at which this transaction is unlocked:
/// Value	Description
/// 0	Not locked
/// < 500000000	Block number at which this transaction is unlocked
/// >= 500000000	UNIX timestamp at which this transaction is unlocked
/// If all TxIn inputs have final (0xffffffff) sequence numbers then lock_time is irrelevant. Otherwise, the transaction may not be added to a block until after lock_time (see NLockTime).
/// 
#[derive(Debug, PartialEq)]
pub struct TransactionMessage {
    pub version: i32,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub witnesses: Vec<Witness>,
    pub lock_time: u32,
}

impl TransactionMessage {
    #[inline]
    pub fn len(&self) -> usize {
        4usize // version
          + 2usize // flag for witness data
          + 4usize // len of inputs
          + self.inputs.iter().map(|i| i.len()).sum::<usize>() // inputs
          + 4usize // len of outputs
          + self.outputs.iter().map(|i| i.len()).sum::<usize>() // outputs
          + self.witnesses.iter().map(|i| i.len()).sum::<usize>() // witnesses
          + 4usize // lock time
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "tx"
    }
}

impl Encode for TransactionMessage {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        self.version.encode(&mut buff)?;
        
        if self.witnesses.len() > 0 {
            [0x00, 0x01].encode(&mut buff)?;
        }
        VarInt::new(self.inputs.len() as u64).encode(&mut buff)?;
        self.inputs.encode(&mut buff)?;

        VarInt::new(self.outputs.len() as u64).encode(&mut buff)?;
        self.outputs.encode(&mut buff)?;

        if self.witnesses.len() > 0 {
            VarInt::new(self.witnesses.len() as u64).encode(&mut buff)?;
            self.witnesses.encode(&mut buff)?;
        }

        self.lock_time.encode(&mut buff)?;
        Ok(())
    }
}