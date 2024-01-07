use std::env;
use std::fs;

use carlito::position;

/*struct UciOption {
    name: String,
    value: String,
}*/



fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: tournament <stockfish reference opponent>");
        return;
    }

    for i in 0..20 {
        
    }

    println!("asdfasdfasdf");
}

const initial_positions: [&str; 100] = [
    "rq2kb1r/p2p1ppp/1pb1pn2/8/2P1P3/P1N5/1PQ1BPPP/R1B1K2R w KQkq - 0 1",
    "rnbqr1k1/pp3ppp/3b4/3p4/3Pn3/2NB1N2/PP3PPP/R1BQ1RK1 w - - 0 1",
    "rnbqr1k1/pp3pb1/3p1npp/2pP4/4P3/2N1BP2/PP1Q2PP/R3KBNR w KQ - 0 1",
    "rnbqr1k1/1p2bppp/2p2n2/p2p4/3P4/P1N1P1P1/1P2NPBP/R1BQ1RK1 w - - 0 1",
    "rnbqkb1r/1p3pp1/p2p1n2/5N1P/4Pp2/2N1B3/PPP4P/R2QKB1R w KQkq - 0 1",
    "rnbqkb1r/1p3p1p/p2p1P2/4p3/4Pp2/2N1B3/PPP2P1P/R2QKB1R w KQkq - 0 1",
    "rnbqk2r/1p2bppp/p2p4/1N1Pp3/2P5/8/PP2BPPP/R1BQK2R w KQkq - 0 1",
    "rnbq1rk1/pp2bppp/3p4/1N1Pp3/2P5/8/PP2BPPP/R1BQK2R w KQ - 0 1",
    "rnbq1rk1/4bppp/p2p1n2/1pp1p3/4P3/1BPP1N1P/PP3PP1/RNBQR1K1 w - - 0 1",
    "rnb2rk1/ppq2pbn/3p2pp/2pPp3/2P1P2B/2N5/PP1NBPPP/R2QK2R w KQ - 0 1",
    "rnb2rk1/pp3ppp/1q6/3p4/1b1N4/1QN3P1/PP2PP1P/R1B1K2R w KQ - 0 1",
    "rnb2rk1/2q1bppp/p2ppn2/1p6/3NPP2/2N1B3/PPP1B1PP/R2Q1R1K w - - 0 1",
    "rnb2rk1/2q1bppp/p2ppn2/1p6/3NP3/1BN2Q2/PPP2PPP/R1B1R1K1 w - - 0 1",
    "rnb2rk1/1pq2pbp/p2ppnp1/8/2PNP3/2NBB3/PP2QPPP/R4RK1 w - - 0 1",
    "rnb1kb1r/2qn1ppp/p3p3/1p2P1B1/3N4/2N5/PPP1Q1PP/R3KB1R w KQkq - 0 1",
    "rnb1k2r/ppn3pp/4pp2/q1P5/3QP3/P1P2P2/6PP/R1B1KBNR w KQkq - 0 1",
    "rn3rk1/pp1bppbp/3p1np1/q2p4/2PN4/2N3PP/PP2PPB1/R1BQ1RK1 w - - 0 1",
    "rn3rk1/p1ppqpp1/1p2p2p/8/2PPb3/4PN2/PP2BPPP/R2QK2R w KQ - 0 1",
    "rn2kb1r/pp3pp1/2p2np1/4q3/8/6N1/PPPB1PPP/R2QKB1R w KQkq - 0 1",
    "rn2k2r/ppqbnppp/4p3/3pP3/1P1p1P2/P4N2/2P3PP/R1BQKB1R w KQkq - 0 1",
    "rn2k2r/pp3ppp/2p1p3/4N3/Pbpqn3/2N2Q2/1P4PP/R1B1KB1R w KQkq - 0 1",
    "rn2k2r/pp1qppbp/6p1/2pP4/4pPn1/2N2N1P/PPP3P1/R1BQK2R w KQkq - 0 1",
    "rn2k2r/5ppp/p1p1pn2/qp1pN3/1bPP4/1PN1P3/P2BQPPP/R3K2R w KQkq - 0 1",
    "rn1qk2r/p3bppp/bpp1p3/3pN3/2PP4/1Pn3P1/P1Q1PPBP/RN2K2R w KQkq - 0 1",
    "rn1qk2r/p1p1nppp/bp1bp3/8/2NPP3/1P4P1/P4PBP/RNBQ1K1R w kq - 0 1",
    "rn1qk2r/4bppp/pp1ppn2/6N1/2PQ4/2N3P1/PP2PPbP/R1BR2K1 w kq - 0 1",
    "rn1q1rk1/ppp1ppbp/1n4p1/8/3PP1b1/1QN1BN2/PP3PPP/R3KB1R w KQ - 0 1",
    "rn1q1rk1/pbpp2pp/1p2p3/4Np2/1bPPB1Q1/6P1/PP1N1P1P/R1B1K2R w KQ - 0 1",
    "rn1q1rk1/pbp2ppp/1p1ppb2/3P4/2P5/2N2NP1/PP1nPPBP/2RQ1RK1 w - - 0 1",
    "rn1q1rk1/pb3ppp/1p3b2/2pp4/3P4/P1N2NP1/1P2PPBP/R2QK2R w KQ - 0 1",
    "rn1q1rk1/pb3ppp/1p2pn2/8/1bBp4/2N1PN2/PP2QPPP/R1BR2K1 w - - 0 1",
    "rn1q1rk1/pb3ppp/1p1b1n2/2ppN3/3P1P2/2NBP3/PP4PP/R1BQ1RK1 w - - 0 1",
    "rn1q1rk1/pb2bppp/1p3n2/2ppN3/3P4/1P4P1/P2NPPBP/R1BQ1RK1 w - - 0 1",
    "rn1q1rk1/pb2bppp/1p2p3/2pn4/3P4/P1NBPN2/1P3PPP/R1BQ1RK1 w - - 0 1",
    "rn1q1rk1/pb1p1ppp/1p2pn2/6B1/2PN4/P1b1P3/1PQ2PPP/R3KB1R w KQ - 0 1",
    "rn1q1rk1/p3bppp/bp3n2/2pp4/3P4/1PN2NPB/P1QBPP1P/R3K2R w KQ - 0 1",
    "rn1q1rk1/1bppbppp/1p2p3/p7/2PP1B2/2n2NP1/PPQ1PPBP/R4RK1 w - - 0 1",
    "rn1q1rk1/1b2bppp/p3pn2/1pP5/8/2NBPN2/PP3PPP/R1BQ1RK1 w - - 0 1",
    "r4rk1/ppqn1ppp/2pb1n2/3p2B1/3P2b1/2PB1N2/PPQN1PPP/R4RK1 w - - 0 1",
    "r4rk1/pp1nbppp/1qp1pn2/8/3PN1b1/2P2NP1/PP3PBP/R1BQR1K1 w - - 0 1",
    "r4rk1/pp1bppbp/2np1np1/q7/2PN4/1PN3P1/PB2PPBP/R2Q1RK1 w - - 0 1",
    "r4rk1/pbpnqppp/1p1ppn2/8/2PP4/P1Q1PN2/1P2BPPP/R1B2RK1 w - - 0 1",
    "r4rk1/pbpnqpbp/1p1ppnp1/8/2PP4/1PN1PN2/PBQ1BPPP/R4RK1 w - - 0 1",
    "r4rk1/p1p1bppp/bpn1pn2/3q4/4NP2/2PP1N2/PPQ1B1PP/R1B1K2R w KQ - 0 1",
    "r4rk1/1bpqbppp/p1np1n2/1p2p3/P3P3/1B1P1N2/1PPN1PPP/R1BQR1K1 w - - 0 1",
    "r3kbnr/pp4p1/1qn1p1p1/2ppP2p/3P2PP/8/PPP1NP2/R1BQKB1R w KQkq - 0 1",
    "r3kbn1/ppqnpppr/2p4p/7P/3P4/5NN1/PPP2PP1/R1BQK2R w KQq - 0 1",
    "r3kb1r/pp3ppp/2n1pn2/q6b/3P4/2N2N1P/PP2BPP1/R1BQ1RK1 w kq - 0 1",
    "r3kb1r/pp1n1ppp/2p1p1b1/q7/2BP2PP/2N5/PPP2P2/R1BQK2R w KQkq - 0 1",
    "r3kb1r/pp1b1ppp/2B1pn2/8/Q1p5/6P1/PP2PP1P/RNBq2K1 w kq - 0 1",
    "r3kb1r/p1pp1p1p/b1p3p1/3nP3/1qP2P2/1P6/P3Q1PP/RNB1KB1R w KQkq - 0 1",
    "r3kb1r/1pqb1pp1/p1nppn1p/8/3NPPP1/2N1B2P/PPP3B1/R2QK2R w KQkq - 0 1",
    "r3k2r/ppp1nppp/2n1q3/1Bb5/3pP3/8/PP1N1PPP/R1BQ1RK1 w kq - 0 1",
    "r3k2r/pp3ppp/2p1pn2/q3nb2/1bBP4/2N5/PPPBQPPP/2KR3R w kq - 0 1",
    "r3k2r/pp1qppbp/2n2np1/3p4/3PP3/1P3N2/P4PPP/RNBQR1K1 w kq - 0 1",
    "r3k2r/pbq1bpp1/1pn1pn1p/2pp4/4P3/2PP1NP1/PP1NQPBP/R1B1R1K1 w kq - 0 1",
    "r3k2r/1pqnbppp/p2pbn2/4p3/4PP2/1NN5/PPP1B1PP/R1BQ1R1K w kq - 0 1",
    "r3k1nr/1pp2pp1/p1pb4/4p2p/4P3/3PBP1P/PPP2P2/RN3RK1 w kq - 0 1",
    "r2qr1k1/pppb1pbp/2np1np1/8/3NP3/2N1B1PP/PPP2PB1/R2Q1RK1 w - - 0 1",
    "r2qr1k1/pp1n1ppp/2pb1n2/3pp3/4P1b1/1P1P1NP1/PBPN1PBP/R3QRK1 w - - 0 1",
    "r2qr1k1/pp1n1ppp/2pb1n2/3pp2b/4P2N/3P2PP/PPPN1PB1/R1B1QRK1 w - - 0 1",
    "r2qkb1r/pp3ppp/1np1p3/P3nb2/3P4/2N2P2/1P2P1PP/R1BQKB1R w KQkq - 0 1",
    "r2qkb1r/pb1p1ppp/4p3/n1p1P3/2P1n3/2Q2NP1/PP1N1P1P/R1B1KB1R w KQkq - 0 1",
    "r2qkb1r/pb1n1ppp/1p2p3/8/3pP3/P1B2N2/1PQ2PPP/R3KB1R w KQkq - 0 1",
    "r2qkb1r/pb1n1pp1/2p1p2p/1p1nP1N1/2pP4/2N3P1/PP3PBP/R1BQ1RK1 w kq - 0 1",
    "r2qkb1r/p2b1ppp/4pn2/1pp5/2QP4/5NP1/PP2PPBP/R1B2RK1 w kq - 0 1",
    "r2qkb1r/1p1b1ppp/p1n1p3/3pN3/3P1B2/2R1P3/PP3PPP/3QKB1R w Kkq - 0 1",
    "r2qkb1r/1b1n1ppp/p3pn2/1p6/3pP3/1B3N2/PP2QPPP/RNBR2K1 w kq - 0 1",
    "r2qkb1r/1b1n1pp1/p2ppn1p/1p4B1/3NP3/1BN5/PPP2PPP/R2QR1K1 w kq - 0 1",
    "r2qk2r/ppp1bppp/2n5/3n4/3P2b1/3B1N2/PP3PPP/RNBQR1K1 w kq - 0 1",
    "r2qk2r/p3bppp/2p1p3/2pn4/5P2/2N5/PPPP2PP/R1BQ1RK1 w kq - 0 1",
    "r2qk2r/p1pp1p2/bp2pn1p/n3P1p1/2PP3B/P1P2P2/6PP/R2QKBNR w KQkq - 0 1",
    "r2qk2r/p1pb1ppp/2p5/2bpP1n1/3N4/5P2/PPP3PP/RNBQ1RK1 w kq - 0 1",
    "r2qk2r/3nppbp/3p1np1/2pP4/4P2P/2N4R/PP3PP1/R1BQ1KN1 w kq - 0 1",
    "r2qk2r/3nbppp/p2pbn2/1p2p3/4P1P1/1NN1BP2/PPPQ3P/R3KB1R w KQkq - 0 1",
    "r2qk2r/3bbppp/p1nppn2/1p4B1/3NP2P/2N2P2/PPPQ2P1/2KR1B1R w kq - 0 1",
    "r2qk2r/2p2ppp/2np1n2/1pb1p2b/4P3/2P2N1P/1PBP1PP1/RNBQ1RK1 w kq - 0 1",
    "r2qk2r/1b1n1ppp/p1pbpn2/1p6/3P4/2NBPN2/PP1B1PPP/R2Q1RK1 w kq - 0 1",
    "r2q1rk1/ppp1ppbp/5np1/3Pn3/2Q1P3/2N2P2/PP2BP1P/R1B1K2R w KQ - 0 1",
    "r2q1rk1/ppp1ppbp/2n3p1/4P3/5Pb1/2nB1N1P/PPP3P1/R1BQ1RK1 w - - 0 1",
    "r2q1rk1/ppp1ppbp/1nn3p1/8/2QPP1b1/2N1BN2/PP2BPPP/3RK2R w K - 0 1",
    "r2q1rk1/ppp1bppp/2n5/3p4/3P2b1/2PB1N2/P1P2PPP/R1BQR1K1 w - - 0 1",
    "r2q1rk1/pp2ppbp/2np2p1/2p2b2/2PP3N/1P4P1/PB1nPPBP/R2Q1RK1 w - - 0 1",
    "r2q1rk1/pp1nbppp/2p2n2/4p2b/4P3/5NPP/PPPN1PB1/R1B1QRK1 w - - 0 1",
    "r2q1rk1/pp1n1ppp/3b1n2/3pp2b/8/3P1NPP/PPPN1PB1/R1BQR1K1 w - - 0 1",
    "r2q1rk1/pp1bp1bp/2pp1np1/2nP1p2/2P5/1PN2NP1/P3PPBP/1RBQ1RK1 w - - 0 1",
    "r2q1rk1/pp1bbppp/3ppn2/8/3nPP2/2N1B3/PPP1B1PP/R3QRK1 w - - 0 1",
    "r2q1rk1/pbpnbpp1/1p3n1p/3p4/3P3B/2NBPN2/PP3PPP/R2Q1RK1 w - - 0 1",
    "r2q1rk1/pb3ppp/1p2pn2/2pp4/1nPP4/1P2PNP1/P4PBP/RN1Q1RK1 w - - 0 1",
    "r2q1rk1/pb2ppbp/1p3np1/2np4/2P5/1PN2NP1/PB2PPBP/R2Q1RK1 w - - 0 1",
    "r2q1rk1/pb1n1ppp/1p1ppn2/2p5/2PP4/P1BBPN2/1P3PPP/R2Q1RK1 w - - 0 1",
    "r2q1rk1/p3bppp/b1p1pn2/3p4/2P5/1P4P1/P1Q1PPBP/RNB2RK1 w - - 0 1",
    "r2q1rk1/p2nbppp/bpp1pn2/3p4/2PP1B2/5NP1/PPQ1PPBP/RN1R2K1 w - - 0 1",
    "r2q1rk1/p1pp1ppp/1pn1pn2/8/2PPb3/1Q3NP1/PP1BPPBP/R4RK1 w - - 0 1",
    "r2q1rk1/bpp2pp1/p1np1n1p/4p3/1PP3b1/P1NP1NP1/1B2PPBP/R2Q1RK1 w - - 0 1",
    "r2q1rk1/1ppb1pp1/pbnp1n1p/1B2p3/3PP2B/2PQ1N2/PP1N1PPP/R4RK1 w - - 0 1",
    "r2q1rk1/1ppb1pb1/n2p1npp/p2Pp3/2P1P2B/2N2N2/PP2BPPP/R2Q1RK1 w - - 0 1",
    "r2q1rk1/1p2bppp/p1npbn2/4p3/4P3/1NN2P2/PPP1B1PP/R1BQ1R1K w - - 0 1",
    "r2q1rk1/1p1nbppp/p2pbn2/4p3/4P3/1NN1BP2/PPP1B1PP/R2Q1RK1 w - - 0 1",
    "r2q1rk1/1p1bbppp/p1nppn2/8/3NP3/2N1B1P1/PPP1QPBP/R4RK1 w - - 0 1"
];