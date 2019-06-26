use bitcoin::{
    util::psbt::serialize::{Deserialize, Serialize},
    Transaction,
};
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use minisketch_rs::Minisketch;
use oddsketch::{Oddsketch, DEFAULT_LEN};
use tokio::codec::{Decoder, Encoder};

use std::io::Error;

pub struct MessageCodec;

pub enum Message {
    Oddsketch(Oddsketch),   // 0 || oddsketch bytes
    Minisketch(Minisketch), // 1 || length (u16) || minisketch
    GetTxs(Vec<u64>),       // 2 || n_ids (u16) || id || .. || id
    Txs(Vec<Transaction>),  // 3 || n_txs (u16) || tx_len (u16) || tx || .. || tx_len (u16) || tx
}

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = Error;
    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            Message::Oddsketch(oddsketch) => {
                let raw = &oddsketch[..];

                dst.reserve(1 + DEFAULT_LEN);
                dst.put_u8(0);
                dst.put_slice(raw);
            }
            Message::Minisketch(minisketch) => {
                let length = minisketch.serialized_size();
                let mut raw = vec![0; length];
                minisketch.serialize(&mut raw).unwrap();

                dst.reserve(3 + length);
                dst.put_u8(1);
                dst.put_u16_be(length as u16);
                dst.put_slice(&raw);
            }
            Message::GetTxs(vec_ids) => {
                let vec_len = vec_ids.len();

                dst.reserve(3 + 8 * vec_len);
                dst.put_u8(2);
                dst.put_u16_be(vec_len as u16);
                for id in &vec_ids {
                    dst.put_u64_be(*id);
                }
            }
            Message::Txs(vec_tx) => {
                let vec_len = vec_tx.len();

                dst.reserve(3);
                dst.put_u8(3);
                dst.put_u16_be(vec_len as u16);

                for tx in &vec_tx {
                    let raw = tx.serialize();
                    let len = raw.len();
                    
                    dst.reserve(2 + len);
                    dst.put_u16_be(len as u16);
                    dst.put_slice(&raw);
                }
            }
        }
        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        let mut buf = src.clone().into_buf();

        match buf.get_u8() {
            0 => {
                if buf.remaining() < DEFAULT_LEN {
                    return Ok(None);
                }

                let mut raw = [0; DEFAULT_LEN];
                buf.copy_to_slice(&mut raw);
                src.advance(DEFAULT_LEN + 1);

                Ok(Some(Message::Oddsketch(Oddsketch::new(raw))))
            }
            1 => {
                if buf.remaining() < 2 {
                    return Ok(None);
                }
                let len = buf.get_u16_be() as usize;
                if buf.remaining() < len {
                    return Ok(None);
                }

                let mut minisketch = Minisketch::try_new(64, 0, len / 8).unwrap();
                let mut raw = vec![0; len];
                buf.copy_to_slice(&mut raw);
                minisketch.deserialize(&raw);

                src.advance(3 + len);

                Ok(Some(Message::Minisketch(minisketch)))
            }
            2 => {
                if buf.remaining() < 2 {
                    return Ok(None);
                }

                let len = buf.get_u16_be() as usize;
                if buf.remaining() < len * 8 {
                    return Ok(None);
                }

                let mut ids = Vec::with_capacity(len);
                for _ in 0..len {
                    let id = buf.get_u64_be();
                    ids.push(id);
                }

                src.advance(3 + len * 8);
                Ok(Some(Message::GetTxs(ids)))
            }
            3 => {
                if buf.remaining() < 2 {
                    return Ok(None);
                }

                let vec_len = buf.get_u16_be() as usize;
                let mut vec_tx = Vec::with_capacity(vec_len);
                let mut total_len = 3;
                for _ in 0..vec_len {
                    if buf.remaining() < 2 {
                        return Ok(None);
                    }

                    let tx_len = buf.get_u16_be() as usize;
                    if buf.remaining() < tx_len {
                        return Ok(None);
                    }

                    total_len += 2 + tx_len;

                    let mut raw_tx = vec![0; tx_len];
                    buf.copy_to_slice(&mut raw_tx);
                    let tx = Transaction::deserialize(&raw_tx).unwrap();
                    vec_tx.push(tx);
                }
                src.advance(total_len);
                Ok(Some(Message::Txs(vec_tx)))
            }
            _ => Err(Error::from_raw_os_error(22)),
        }
    }
}
