//! This fork's strings (FORK.md): Labs › Send to endpoint's card and the
//! sidebar's Hex on GitHub, in every language Hex has. They live here, not
//! in upstream's tables, so those files stay the same as upstream's.

/// Columns: Polish, Chinese (Simplified), Japanese, German, Spanish, Ukrainian.
#[rustfmt::skip]
pub(super) const TRANSLATIONS: &[(&str, [&str; 6])] = &[
    ("Send to endpoint", ["Wyślij do punktu końcowego", "发送到端点", "エンドポイントに送信", "An Endpunkt senden", "Enviar al punto de conexión", "Надсилати на кінцеву точку"]),
    ("Start with an endpoint's name to send it the rest: \"Restage, open portfolio\"", ["Zacznij od nazwy punktu końcowego, aby wysłać mu resztę: \"Restage, open portfolio\"", "以端点名称开头，即可把其余内容发送给它：\"Restage, open portfolio\"", "エンドポイント名で始めると、残りをそこへ送ります：\"Restage, open portfolio\"", "Beginne mit dem Namen eines Endpunkts, um ihm den Rest zu senden: \"Restage, open portfolio\"", "Empieza con el nombre de un punto de conexión para enviarle el resto: \"Restage, open portfolio\"", "Почніть із назви кінцевої точки, щоб надіслати їй решту: \"Restage, open portfolio\""]),
    ("Endpoint URL", ["Adres URL punktu końcowego", "端点 URL", "エンドポイントの URL", "Endpunkt-URL", "URL del punto de conexión", "URL кінцевої точки"]),
    ("Bearer token", ["Token bearer", "Bearer 令牌", "Bearer トークン", "Bearer-Token", "Token bearer", "Токен bearer"]),
    ("JSON body", ["Treść JSON", "JSON 正文", "JSON 本文", "JSON-Body", "Cuerpo JSON", "Тіло JSON"]),
    ("Add endpoint", ["Dodaj punkt końcowy", "添加端点", "エンドポイントを追加", "Endpunkt hinzufügen", "Añadir punto de conexión", "Додати кінцеву точку"]),
    ("Also paste the dictation", ["Wklej też dyktowanie", "同时粘贴听写内容", "ディクテーションも貼り付ける", "Diktat auch einfügen", "Pegar también el dictado", "Також вставляти диктування"]),
    ("Hex on GitHub", ["Hex na GitHubie", "GitHub 上的 Hex", "GitHub の Hex", "Hex auf GitHub", "Hex en GitHub", "Hex на GitHub"]),
];

#[rustfmt::skip]
pub(super) const FRENCH: &[(&str, &str)] = &[
    ("Send to endpoint", "Envoyer à un point de terminaison"),
    ("Start with an endpoint's name to send it the rest: \"Restage, open portfolio\"", "Commencez par le nom d'un point de terminaison pour lui envoyer le reste : \"Restage, open portfolio\""),
    ("Endpoint URL", "URL du point de terminaison"),
    ("Bearer token", "Jeton bearer"),
    ("JSON body", "Corps JSON"),
    ("Add endpoint", "Ajouter un point de terminaison"),
    ("Also paste the dictation", "Coller aussi la dictée"),
    ("Hex on GitHub", "Hex sur GitHub"),
];

#[rustfmt::skip]
pub(super) const PORTUGUESE: &[(&str, &str)] = &[
    ("Send to endpoint", "Enviar ao endpoint"),
    ("Start with an endpoint's name to send it the rest: \"Restage, open portfolio\"", "Comece pelo nome de um endpoint para lhe enviar o resto: \"Restage, open portfolio\""),
    ("Endpoint URL", "URL do endpoint"),
    ("Bearer token", "Token bearer"),
    ("JSON body", "Corpo JSON"),
    ("Add endpoint", "Adicionar endpoint"),
    ("Also paste the dictation", "Também colar o ditado"),
    ("Hex on GitHub", "Hex no GitHub"),
];

#[rustfmt::skip]
pub(super) const RUSSIAN: &[(&str, &str)] = &[
    ("Send to endpoint", "Отправлять на конечную точку"),
    ("Start with an endpoint's name to send it the rest: \"Restage, open portfolio\"", "Начните с названия конечной точки, чтобы отправить ей остальное: \"Restage, open portfolio\""),
    ("Endpoint URL", "URL конечной точки"),
    ("Bearer token", "Токен bearer"),
    ("JSON body", "Тело JSON"),
    ("Add endpoint", "Добавить конечную точку"),
    ("Also paste the dictation", "Также вставлять диктовку"),
    ("Hex on GitHub", "Hex на GitHub"),
];
