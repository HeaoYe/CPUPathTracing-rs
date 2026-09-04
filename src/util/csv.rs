pub fn csv_column(headers: &csv::StringRecord, name: &str) -> Result<usize, std::io::Error> {
    headers
        .iter()
        .position(|header| header == name)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("missing CSV header: {name}"),
            )
        })
}

pub fn parse_csv<T>(value: &str) -> Result<T, std::io::Error>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    value
        .parse()
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}
