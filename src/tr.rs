#[cfg(all(feature = "en", not(feature = "fr")))]
mod texts {
    pub const TODAY: &str = "Today";
    pub const WEEKDAYS: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
    pub const MONTH_NAMES: [&str; 12] = [
        "January", "February", "March", "April",
        "Mei", "June", "July", "August",
        "September", "October", "November", "December"
    ];
}

#[cfg(feature = "fr")]
mod texts {
    pub const TODAY: &str = "Aujourd'hui";
    pub const WEEKDAYS: [&str; 7] = ["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche"];
    pub const MONTH_NAMES: [&str; 12] = [
        "Janvier", "Février", "Mars", "Avril",
        "Mai", "Juin", "Juillet", "Août",
        "Septembre", "Octobre", "Novembre", "Décembre"
    ];
}

pub use texts::*;