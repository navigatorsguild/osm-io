use anyhow::anyhow;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub(crate) enum UserStatus {
    Pending,
    Active,
    Confirmed,
    Suspended,
    Deleted,
}

impl TryFrom<&str> for UserStatus {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => {
                Ok(UserStatus::Pending)
            }
            "active" => {
                Ok(UserStatus::Active)
            }
            "confirmed" => {
                Ok(UserStatus::Confirmed)
            }
            "suspended" => {
                Ok(UserStatus::Suspended)
            }
            "deleted" => {
                Ok(UserStatus::Deleted)
            }
            _ => {
                Err(anyhow!("Unknown user status: {}", value))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum FormatEnum {
    Html,
    Markdown,
    Text,
}

impl TryFrom<&str> for FormatEnum {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "html" => {
                Ok(FormatEnum::Html)
            }
            "markdown" => {
                Ok(FormatEnum::Markdown)
            }
            "text" => {
                Ok(FormatEnum::Text)
            }
            _ => {
                Err(anyhow!("Unknown format: {}", value))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct UserRecord {
    email: String,
    id: i64,
    pass_crypt: String,
    creation_time: NaiveDateTime,
    display_name: String,
    data_public: bool,
    description: String,
    home_lat: Option<f64>,
    home_lon: Option<f64>,
    home_zoom: i16,
    pass_salt: Option<String>,
    email_valid: bool,
    new_email: Option<String>,
    creation_ip: Option<String>,
    languages: Option<String>,
    status: UserStatus,
    terms_agreed: Option<NaiveDateTime>,
    consider_pd: bool,
    auth_uid: Option<String>,
    preferred_editor: Option<String>,
    terms_seen: bool,
    description_format: FormatEnum,
    changesets_count: i32,
    traces_count: i32,
    diary_entries_count: i32,
    image_use_gravatar: bool,
    auth_provider: Option<String>,
    home_tile: Option<i64>,
    tou_agreed: Option<NaiveDateTime>,
}

impl UserRecord {
    pub(crate) fn new(
        email: String,
        id: i64,
        pass_crypt: String,
        creation_time: NaiveDateTime,
        display_name: String,
        data_public: bool,
        description: String,
        home_lat: Option<f64>,
        home_lon: Option<f64>,
        home_zoom: i16,
        pass_salt: Option<String>,
        email_valid: bool,
        new_email: Option<String>,
        creation_ip: Option<String>,
        languages: Option<String>,
        status: UserStatus,
        terms_agreed: Option<NaiveDateTime>,
        consider_pd: bool,
        auth_uid: Option<String>,
        preferred_editor: Option<String>,
        terms_seen: bool,
        description_format: FormatEnum,
        changesets_count: i32,
        traces_count: i32,
        diary_entries_count: i32,
        image_use_gravatar: bool,
        auth_provider: Option<String>,
        home_tile: Option<i64>,
        tou_agreed: Option<NaiveDateTime>,
    ) -> UserRecord {
        UserRecord {
            email,
            id,
            pass_crypt,
            creation_time,
            display_name,
            data_public,
            description,
            home_lat,
            home_lon,
            home_zoom,
            pass_salt,
            email_valid,
            new_email,
            creation_ip,
            languages,
            status,
            terms_agreed,
            consider_pd,
            auth_uid,
            preferred_editor,
            terms_seen,
            description_format,
            changesets_count,
            traces_count,
            diary_entries_count,
            image_use_gravatar,
            auth_provider,
            home_tile,
            tou_agreed,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn email(&self) -> &String {
        &self.email
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    #[allow(dead_code)]
    pub(crate) fn pass_crypt(&self) -> &String {
        &self.pass_crypt
    }

    #[allow(dead_code)]
    pub(crate) fn creation_time(&self) -> &NaiveDateTime {
        &self.creation_time
    }

    pub(crate) fn display_name(&self) -> &String {
        &self.display_name
    }

    #[allow(dead_code)]
    pub(crate) fn data_public(&self) -> bool {
        self.data_public
    }

    #[allow(dead_code)]
    pub(crate) fn description(&self) -> &String {
        &self.description
    }

    #[allow(dead_code)]
    pub(crate) fn home_lat(&self) -> Option<f64> {
        self.home_lat
    }

    #[allow(dead_code)]
    pub(crate) fn home_lon(&self) -> Option<f64> {
        self.home_lon
    }

    #[allow(dead_code)]
    pub(crate) fn home_zoom(&self) -> i16 {
        self.home_zoom
    }

    #[allow(dead_code)]
    pub(crate) fn pass_salt(&self) -> Option<&String> {
        self.pass_salt.as_ref()
    }

    #[allow(dead_code)]
    pub(crate) fn email_valid(&self) -> bool {
        self.email_valid
    }

    #[allow(dead_code)]
    pub(crate) fn new_email(&self) -> Option<&String> {
        self.new_email.as_ref()
    }

    #[allow(dead_code)]
    pub(crate) fn creation_ip(&self) -> Option<&String> {
        self.creation_ip.as_ref()
    }

    #[allow(dead_code)]
    pub(crate) fn languages(&self) -> Option<&String> {
        self.languages.as_ref()
    }

    #[allow(dead_code)]
    pub(crate) fn status(&self) -> &UserStatus {
        &self.status
    }

    #[allow(dead_code)]
    pub(crate) fn terms_agreed(&self) -> &Option<NaiveDateTime> {
        &self.terms_agreed
    }

    #[allow(dead_code)]
    pub(crate) fn consider_pd(&self) -> bool {
        self.consider_pd
    }

    #[allow(dead_code)]
    pub(crate) fn auth_uid(&self) -> Option<&String> {
        self.auth_uid.as_ref()
    }

    #[allow(dead_code)]
    pub(crate) fn preferred_editor(&self) -> Option<&String> {
        self.preferred_editor.as_ref()
    }

    #[allow(dead_code)]
    pub(crate) fn terms_seen(&self) -> bool {
        self.terms_seen
    }

    #[allow(dead_code)]
    pub(crate) fn description_format(&self) -> &FormatEnum {
        &self.description_format
    }

    #[allow(dead_code)]
    pub(crate) fn changesets_count(&self) -> i32 {
        self.changesets_count
    }

    #[allow(dead_code)]
    pub(crate) fn traces_count(&self) -> i32 {
        self.traces_count
    }

    #[allow(dead_code)]
    pub(crate) fn diary_entries_count(&self) -> i32 {
        self.diary_entries_count
    }

    #[allow(dead_code)]
    pub(crate) fn image_use_gravatar(&self) -> bool {
        self.image_use_gravatar
    }

    #[allow(dead_code)]
    pub(crate) fn auth_provider(&self) -> Option<&String> {
        self.auth_provider.as_ref()
    }

    #[allow(dead_code)]
    pub(crate) fn home_tile(&self) -> Option<i64> {
        self.home_tile
    }

    #[allow(dead_code)]
    pub(crate) fn tou_agreed(&self) -> &Option<NaiveDateTime> {
        &self.tou_agreed
    }
}
