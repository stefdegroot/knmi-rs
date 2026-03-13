// pub mod arome;
pub mod grib;

pub trait Model {
    fn load_model (&self) -> ();

    fn update_model (&self) -> ();
}