mod lb_factory;
mod lb_pair;
mod lb_quoter;
mod lb_router;

pub fn main() -> std::io::Result<()> {
    lb_factory::main()?;
    lb_pair::main()?;
    lb_quoter::main()?;
    lb_router::main()?;

    Ok(())
}
