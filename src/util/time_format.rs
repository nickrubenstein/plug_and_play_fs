use time::{
    OffsetDateTime, UtcOffset, format_description
};

pub fn format_time<T: Into<OffsetDateTime>>(dt: T, format: Option<&str>) -> String {
  let default_format = "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour \
  sign:mandatory]:[offset_minute]:[offset_second]";
  let format = format_description::parse(format.unwrap_or(default_format));
  let mut date_time: OffsetDateTime = dt.into();
  date_time = date_time.to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
  date_time
      .format(&format.unwrap())
      .unwrap_or(String::from("Unknown"))
}

pub fn now(format: Option<&str>) -> String {
    let time = std::time::SystemTime::now();
    format_time(time, format)
}
