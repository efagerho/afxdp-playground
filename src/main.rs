use xdpsock::{xsk::Xsk2, BindFlags, SocketConfig, SocketConfigBuilder, UmemConfig, UmemConfigBuilder, XdpFlags};
use clap::Parser;
use std::thread;

const XDP_QUEUE_SIZE: u32 = 8192;

#[derive(Clone, Debug, Parser)]
struct Opt {
    /// Iface name
    #[clap(long)]
    iface: String,
    /// NIC combined queues count
    #[clap(long)]
    queues: u32
}

fn nic_queue_io_thread(if_name: String, queue: u32, umem_config: UmemConfig, socket_config: SocketConfig) {
    let n_tx_frames = umem_config.frame_count() / 2;

    let mut xsk = Xsk2::new(&if_name, queue, umem_config, socket_config, n_tx_frames as usize);

    loop {
        let (pkt, len) = xsk.recv().expect("Receive failure");
    }
}

fn main() {
    env_logger::init();
    let opts: Opt = Opt::parse();

    let umem_config = UmemConfigBuilder::new()
        .frame_count(XDP_QUEUE_SIZE)
        .comp_queue_size(XDP_QUEUE_SIZE / 2)
        .fill_queue_size(XDP_QUEUE_SIZE / 2)
        .build()
        .unwrap();

    let socket_config = SocketConfigBuilder::new()
        .tx_queue_size(XDP_QUEUE_SIZE / 2)
        .rx_queue_size(XDP_QUEUE_SIZE / 2)
        .bind_flags(BindFlags::XDP_COPY)
        .xdp_flags(XdpFlags::XDP_FLAGS_SKB_MODE)
        .build()
        .unwrap();

    let mut handles = vec![];

    for queue in 0..opts.queues {
        let umem_config = umem_config.clone();
        let socket_config = socket_config.clone();
        let if_name = opts.iface.clone();

        let handle = thread::spawn(move || {
            log::info!("Starting thread listening on NIC queue {queue}");
            nic_queue_io_thread(if_name, queue, umem_config, socket_config);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("failed to join on io thread handle");
    }
}

