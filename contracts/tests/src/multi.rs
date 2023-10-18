pub mod lb_factory {
    use lb_factory;
    multi_derive::implement_multi!(LbFactory, lb_factory);
}

pub mod lb_pair {
    use lb_pair;
    multi_derive::implement_multi!(LbPair, lb_pair);
}

pub mod lb_token {
    use lb_token;
    multi_derive::implement_multi!(LbToken, lb_token);
}
