use cdtext::CDText;

fn main() {
    let Some(filename) = std::env::args().skip(1).next() else {
        eprintln!("No filename provided!");

        std::process::exit(1);
    };

    let data = match std::fs::read(filename) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read the file: {e:?}");

            std::process::exit(1);
        }
    };

    let cdtext = CDText::from_data_with_length(&data);

    let data: Vec<cdtext::CDTextEntry> = cdtext.parse();

    for i in data {
        let displayable_track = match i.track_number {
            cdtext::CDTextTrackNumber::WholeAlbum => {
                format!("Album")
            },
            cdtext::CDTextTrackNumber::Track(nr) => {
                format!("Track #{nr}")
            },
        };

        println!("{displayable_track}: {:?}: {:?}", i.entry_type, i.data);
    }
}
