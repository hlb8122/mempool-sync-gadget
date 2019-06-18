use crate::mempool::Mempool;

enum State {
    Idle,
    Pull,
    Push
}

struct MempoolGadget {
    mempool: Mutex<Mempool>,
    state: Mutex<State>
}

enum Messages {

}