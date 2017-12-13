use chrono::NaiveDateTime;

use nom::{be_i8, be_u8, be_u16, be_u32, be_i64, IResult};


#[derive(Debug, PartialEq, Eq)]
pub struct CHTHeader {
    pub event_type: u16,
    pub len: u32,
    pub cht_id: u32,
    pub data: Option<Vec<StatSta>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StatSta {
    pub mac: Vec<u8>,        // Mac of the station
    pub snr: u8,             // SNR
    pub signal: i8,          // Signal power
    pub noise: i8,           // Noise power
    pub rx_packets: u32,     // Received packets
    pub tx_packets: u32,     // Transmitted packets
    pub tx_rate: u32,        // TX rate
    pub tx_mcs: i8,          // TX MCS
    pub tx_flags: WifiFlags, // [x,x,x,x,is_vht,is_ht,is_short_gi,is40mhz]
    pub tx_mhz: u8,          // TX Bandwidth
    pub tx_vht_nss: u8,      // TX VHT NSS
    pub rx_rate: u32,        // RX rate
    pub rx_mcs: i8,          // RX MCS
    pub rx_flags: WifiFlags, // [x,x,x,x,is_vht,is_ht,is_short_gi,is40mhz]
    pub rx_mhz: u8,          // RX Bandwidth
    pub rx_vht_nss: u8,      // RX VHT NSS
    pub rx_bytes: u32,       // Received bytes
    pub tx_bytes: u32,       // Transmitted bytes
    pub rx_retries: u32,     // TX bytes retries
    pub rx_failed: u32,      // TX bytes failed
    pub timestamp: NaiveDateTime, // Timestamp
}

/*
pub struct Mac(Vec<u8>);

impl fmt::UpperHex for Mac {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = self.0;
        write!(f, "{:X}", val) // delegate to i32's implementation
    }
}
*/

bitflags! {
    pub struct WifiFlags: u8 {
        const IS_40_MHZ = 0b00000001;
        const IS_SHORT_GI = 0b00000010;
        const IS_HT = 0b00000100;
        const IS_VHT = 0b00001000;
    }
}


fn stat_sta(input: &[u8]) -> IResult<&[u8], StatSta> {
    do_parse!(input,
        mac: take!(6) >>
        snr: be_u8 >>
        signal: be_i8 >>
        noise: be_i8 >>
        rx_packets: be_u32 >>
        tx_packets: be_u32 >>
        tx_rate: be_u32 >>
        tx_mcs: be_i8 >>
        tx_flags: be_u8 >>
        tx_mhz: be_u8 >>
        tx_vht_nss: be_u8 >>
        rx_rate: be_u32 >>
        rx_mcs: be_i8 >>
        rx_flags: be_u8 >>
        rx_mhz: be_u8 >>
        rx_vht_nss: be_u8 >>
        rx_bytes: be_u32 >>
        tx_bytes: be_u32 >>
        rx_retries: be_u32 >>
        rx_failed: be_u32 >>
        timestamp: be_i64 >>
        (
            StatSta {
                mac: mac.into(),
                snr: snr,
                signal: signal,
                noise: noise,
                rx_packets: rx_packets,
                tx_packets: tx_packets,
                tx_rate: tx_rate,
                tx_mcs: tx_mcs,
                tx_flags: WifiFlags::from_bits(tx_flags).unwrap(),
                tx_mhz: tx_mhz,
                tx_vht_nss: tx_vht_nss,
                rx_rate: rx_rate,
                rx_mcs: rx_mcs,
                rx_flags: WifiFlags::from_bits(rx_flags).unwrap(),
                rx_mhz: rx_mhz,
                rx_vht_nss: rx_vht_nss,
                rx_bytes: rx_bytes,
                tx_bytes: tx_bytes,
                rx_retries: rx_retries,
                rx_failed: rx_failed,
                timestamp: NaiveDateTime::from_timestamp(timestamp / 1000, 0),
            }
       )
    )
}

fn vec_stat_sta(input: &[u8], size: usize) -> IResult<&[u8], Vec<StatSta>> {
    do_parse!(input,
        num_stas: be_u8 >>
        stas: count!(stat_sta, num_stas as usize) >>
        (
            stas
        )
    )
}

pub fn parser(input: &[u8]) -> IResult<&[u8], CHTHeader> {
    do_parse!(input,
        event_type: be_u16 >>
        len: be_u32 >>
        cht_id: be_u32 >>
        data:
            cond!(
                event_type == 0x1cba,
                call!(vec_stat_sta, len as usize)
            ) >>
        (
            CHTHeader {
                event_type: event_type,
                len: len,
                cht_id: cht_id,
                data: data
            }
        )
    )
}
