use anyhow::{Context, Result};
use clap::Clap;
use data_encoding::BASE32_NOPAD;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::info;
use log::LevelFilter;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use rio_api::formatter::TriplesFormatter;
use rio_api::model::{BlankNode, Literal, NamedNode, Triple};
use rio_turtle::NTriplesFormatter;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::max())
        .init();
    let opts = Opts::parse();

    let output_file = File::create(opts.path_to).context("Could not create output file")?;
    let output_writer = BufWriter::new(output_file);
    let output_writer = GzEncoder::new(output_writer, Compression::best());
    let mut formatter = NTriplesFormatter::new(output_writer);

    write_rdf(
        "Badges",
        &mut formatter,
        write_badge,
        opts.path_from.join("Badges.xml"),
    )?;

    write_rdf(
        "Comments",
        &mut formatter,
        write_comment,
        opts.path_from.join("Comments.xml"),
    )?;

    write_rdf(
        "Posts",
        &mut formatter,
        write_post,
        opts.path_from.join("Posts.xml"),
    )?;

    write_rdf(
        "PostHistory",
        &mut formatter,
        write_posthistory,
        opts.path_from.join("PostHistory.xml"),
    )?;

    write_rdf(
        "PostLinks",
        &mut formatter,
        write_postlink,
        opts.path_from.join("PostLinks.xml"),
    )?;

    write_rdf(
        "Tags",
        &mut formatter,
        write_tag,
        opts.path_from.join("Tags.xml"),
    )?;

    write_rdf(
        "Users",
        &mut formatter,
        write_user,
        opts.path_from.join("Users.xml"),
    )?;

    formatter.finish();

    Ok(())
}

#[derive(Clap)]
struct Opts {
    #[clap(name = "xml_directory")]
    path_from: PathBuf,
    #[clap(name = "output.rdf.gz")]
    path_to: PathBuf,
}

fn write_rdf<W: Write, P: AsRef<Path>>(
    name: &str,
    formatter: &mut NTriplesFormatter<W>,
    writer: impl Fn(&mut NTriplesFormatter<W>, &BytesStart) -> Result<()>,
    path: P,
) -> Result<()> {
    info!("{}: started", name);
    let mut buf = Vec::new();

    let mut reader = Reader::from_file(path)?;
    reader.trim_text(true);

    let mut count = 0usize;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Empty(e)) => {
                writer(formatter, &e)?;
                count += 1;
                if count % 100000 == 0 {
                    info!("{}: count: {}", name, count);
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => (),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
        buf.clear();
    }

    info!("{}: count: {}", name, count);
    info!("{}: finished", name);

    Ok(())
}

fn write_badge<W: Write>(formatter: &mut NTriplesFormatter<W>, e: &BytesStart) -> Result<()> {
    let mut id = None;
    let mut user_id = None;
    let mut name = None;
    let mut date = None;
    let mut class = None;
    let mut tag_based = None;

    for attribute in e.attributes() {
        let attribute = attribute.context("could not parse attribute")?;
        match attribute.key {
            b"Id" => &mut id,
            b"UserId" => &mut user_id,
            b"Name" => &mut name,
            b"Date" => &mut date,
            b"Class" => &mut class,
            b"TagBased" => &mut tag_based,
            _ => continue,
        }
        .replace(parse_attribute(attribute)?);
    }

    let mut id = id.context("`Id` not found in attributes")?;
    id.insert(0, 'b');

    let mut user_id = user_id.context("`UserId` not found in attributes")?;
    user_id.insert(0, 'u');
    let triple = id_to_id(&id, "badge.user", &user_id);
    formatter.format(&triple)?;

    let name = name.context("`Name` not found in attributes")?;
    let triple = id_to_str(&id, "badge.name", &name);
    formatter.format(&triple)?;

    let date = date.context("`Date` not found in attributes")?;
    let triple = id_to_str(&id, "badge.date", &date);
    formatter.format(&triple)?;

    let class = class.context("`Class` not found in attributes")?;
    let triple = id_to_str(&id, "badge.class", &class);
    formatter.format(&triple)?;

    let mut tag_based = tag_based.context("`TagBased` not found in attributes")?;
    tag_based.make_ascii_lowercase();
    let triple = id_to_str(&id, "badge.tag_based", &tag_based);
    formatter.format(&triple)?;

    Ok(())
}

fn write_comment<W: Write>(formatter: &mut NTriplesFormatter<W>, e: &BytesStart) -> Result<()> {
    let mut id = None;
    let mut post_id = None;
    let mut score = None;
    let mut text = None;
    let mut creation_date = None;
    let mut user_id = None;
    let mut user_display_name = None;
    let mut content_license = None;

    for attribute in e.attributes() {
        let attribute = attribute.context("could not parse attribute")?;
        match attribute.key {
            b"Id" => &mut id,
            b"PostId" => &mut post_id,
            b"Score" => &mut score,
            b"Text" => &mut text,
            b"CreationDate" => &mut creation_date,
            b"UserId" => &mut user_id,
            b"UserDisplayName" => &mut user_display_name,
            b"ContentLicense" => &mut content_license,
            _ => continue,
        }
        .replace(parse_attribute(attribute)?);
    }

    let mut id = id.context("`Id` not found in attributes")?;
    id.insert(0, 'c');

    let mut post_id = post_id.context("`PostId` not found in attributes")?;
    post_id.insert(0, 'p');
    let triple = id_to_id(&id, "comment.post", &post_id);
    formatter.format(&triple)?;

    let score = score.context("`Score` not found in attributes")?;
    let triple = id_to_str(&id, "comment.score", &score);
    formatter.format(&triple)?;

    let text = text.context("`Text` not found in attributes")?;
    let triple = id_to_str(&id, "comment.text", &text);
    formatter.format(&triple)?;

    let creation_date = creation_date.context("`CreationDate` not found in attributes")?;
    let triple = id_to_str(&id, "comment.creation_date", &creation_date);
    formatter.format(&triple)?;

    if let Some(mut user_id) = user_id {
        user_id.insert(0, 'u');
        let triple = id_to_id(&id, "comment.user", &user_id);
        formatter.format(&triple)?;
    }

    if let Some(user_display_name) = user_display_name {
        let triple = id_to_str(&id, "comment.user_display_name", &user_display_name);
        formatter.format(&triple)?;
    }

    let content_license = content_license.context("`ContentLicense` not found in attributes")?;
    let triple = id_to_str(&id, "comment.content_license", &content_license);
    formatter.format(&triple)?;

    Ok(())
}

fn write_post<W: Write>(formatter: &mut NTriplesFormatter<W>, e: &BytesStart) -> Result<()> {
    let mut id = None;
    let mut type_ = None;
    let mut accepted_answer_id = None;
    let mut parent_id = None;
    let mut creation_date = None;
    let mut deletion_date = None;
    let mut score = None;
    let mut view_count = None;
    let mut body = None;
    let mut owner_id = None;
    let mut owner_display_name = None;
    let mut last_editor_id = None;
    let mut last_editor_display_name = None;
    let mut last_edit_date = None;
    let mut last_activity_date = None;
    let mut title = None;
    let mut tags = None;
    let mut answer_count = None;
    let mut comment_count = None;
    let mut favorite_count = None;
    let mut closed_date = None;
    let mut community_owned_date = None;
    let mut content_license = None;

    for attribute in e.attributes() {
        let attribute = attribute.context("could not parse attribute")?;
        match attribute.key {
            b"Id" => &mut id,
            b"PostTypeId" => &mut type_,
            b"AcceptedAnswerId" => &mut accepted_answer_id,
            b"ParentId" => &mut parent_id,
            b"CreationDate" => &mut creation_date,
            b"DeletionDate" => &mut deletion_date,
            b"Score" => &mut score,
            b"ViewCount" => &mut view_count,
            b"Body" => &mut body,
            b"OwnerUserId" => &mut owner_id,
            b"OwnerDisplayName" => &mut owner_display_name,
            b"LastEditorUserId" => &mut last_editor_id,
            b"LastEditorDisplayName" => &mut last_editor_display_name,
            b"LastEditDate" => &mut last_edit_date,
            b"LastActivityDate" => &mut last_activity_date,
            b"Title" => &mut title,
            b"Tags" => &mut tags,
            b"AnswerCount" => &mut answer_count,
            b"CommentCount" => &mut comment_count,
            b"FavoriteCount" => &mut favorite_count,
            b"ClosedDate" => &mut closed_date,
            b"CommunityOwnedDate" => &mut community_owned_date,
            b"ContentLicense" => &mut content_license,
            _ => continue,
        }
        .replace(parse_attribute(attribute)?);
    }

    let mut id = id.context("`Id` not found in attributes")?;
    id.insert(0, 'p');

    let type_ = type_.context("`PostTypeId` not found in attributes")?;
    let triple = id_to_str(&id, "post.type", &type_);
    formatter.format(&triple)?;

    if let Some(mut accepted_answer_id) = accepted_answer_id {
        accepted_answer_id.insert(0, 'p');
        let triple = id_to_id(&id, "post.accepted_answer", &accepted_answer_id);
        formatter.format(&triple)?;
    }

    if let Some(mut parent_id) = parent_id {
        parent_id.insert(0, 'p');
        let triple = id_to_id(&id, "post.parent", &parent_id);
        formatter.format(&triple)?;
    }

    let creation_date = creation_date.context("`CreationDate` not found in attributes")?;
    let triple = id_to_str(&id, "post.creation_date", &creation_date);
    formatter.format(&triple)?;

    if let Some(deletion_date) = deletion_date {
        let triple = id_to_str(&id, "post.deletion_date", &deletion_date);
        formatter.format(&triple)?;
    }

    let score = score.context("`Score` not found in attributes")?;
    let triple = id_to_str(&id, "post.score", &score);
    formatter.format(&triple)?;

    if let Some(view_count) = view_count {
        let triple = id_to_str(&id, "post.view_count", &view_count);
        formatter.format(&triple)?;
    }

    let body = body.context("`Body` not found in attributes")?;
    let triple = id_to_str(&id, "post.body", &body);
    formatter.format(&triple)?;

    if let Some(mut owner_id) = owner_id {
        owner_id.insert(0, 'u');
        let triple = id_to_id(&id, "post.owner", &owner_id);
        formatter.format(&triple)?;
    }

    if let Some(owner_display_name) = owner_display_name {
        let triple = id_to_str(&id, "post.owner_display_name", &owner_display_name);
        formatter.format(&triple)?;
    }

    if let Some(mut last_editor_id) = last_editor_id {
        last_editor_id.insert(0, 'u');
        let triple = id_to_id(&id, "post.last_editor", &last_editor_id);
        formatter.format(&triple)?;
    }

    if let Some(last_editor_display_name) = last_editor_display_name {
        let triple = id_to_str(
            &id,
            "post.last_editor_display_name",
            &last_editor_display_name,
        );
        formatter.format(&triple)?;
    }

    if let Some(last_edit_date) = last_edit_date {
        let triple = id_to_str(&id, "post.last_edit_date", &last_edit_date);
        formatter.format(&triple)?;
    }

    if let Some(last_activity_date) = last_activity_date {
        let triple = id_to_str(&id, "post.last_activity_date", &last_activity_date);
        formatter.format(&triple)?;
    }

    if let Some(title) = title {
        let triple = id_to_str(&id, "post.title", &title);
        formatter.format(&triple)?;
    }

    if let Some(mut tags) = tags {
        tags.remove(0);
        tags.pop();
        for tag in tags.split("><") {
            let mut tag_id = BASE32_NOPAD.encode(tag.as_bytes());
            tag_id.insert(0, 't');
            let triple = id_to_id(&id, "post.tags", &tag_id);
            formatter.format(&triple)?;
        }
    }

    if let Some(answer_count) = answer_count {
        let triple = id_to_str(&id, "post.answer_count", &answer_count);
        formatter.format(&triple)?;
    }

    if let Some(comment_count) = comment_count {
        let triple = id_to_str(&id, "post.comment_count", &comment_count);
        formatter.format(&triple)?;
    }

    if let Some(favorite_count) = favorite_count {
        let triple = id_to_str(&id, "post.favorite_count", &favorite_count);
        formatter.format(&triple)?;
    }

    if let Some(closed_date) = closed_date {
        let triple = id_to_str(&id, "post.closed_date", &closed_date);
        formatter.format(&triple)?;
    }

    if let Some(community_owned_date) = community_owned_date {
        let triple = id_to_str(&id, "post.community_owned_date", &community_owned_date);
        formatter.format(&triple)?;
    }

    let content_license = content_license.context("`ContentLicense` not found in attributes")?;
    let triple = id_to_str(&id, "post.content_license", &content_license);
    formatter.format(&triple)?;

    Ok(())
}

fn write_posthistory<W: Write>(formatter: &mut NTriplesFormatter<W>, e: &BytesStart) -> Result<()> {
    let mut id = None;
    let mut type_ = None;
    let mut post_id = None;
    let mut revision_guid = None;
    let mut creation_date = None;
    let mut user_id = None;
    let mut user_display_name = None;
    let mut comment = None;
    let mut text = None;
    let mut content_license = None;

    for attribute in e.attributes() {
        let attribute = attribute.context("could not parse attribute")?;
        match attribute.key {
            b"Id" => &mut id,
            b"PostHistoryTypeId" => &mut type_,
            b"PostId" => &mut post_id,
            b"RevisionGUID" => &mut revision_guid,
            b"CreationDate" => &mut creation_date,
            b"UserId" => &mut user_id,
            b"UserDisplayName" => &mut user_display_name,
            b"Comment" => &mut comment,
            b"Text" => &mut text,
            b"ContentLicense" => &mut content_license,
            _ => continue,
        }
        .replace(parse_attribute(attribute)?);
    }

    let mut id = id.context("`Id` not found in attributes")?;
    id.insert(0, 'h');

    let type_ = type_.context("`PostHistoryTypeId` not found in attributes")?;
    let triple = id_to_str(&id, "posthistory.type", &type_);
    formatter.format(&triple)?;

    let mut post_id = post_id.context("`PostHistoryTypeId` not found in attributes")?;
    post_id.insert(0, 'p');
    let triple = id_to_id(&id, "posthistory.post", &post_id);
    formatter.format(&triple)?;

    let revision_guid = revision_guid.context("`RevisionGUID` not found in attributes")?;
    let triple = id_to_str(&id, "posthistory.revision_guid", &revision_guid);
    formatter.format(&triple)?;

    let creation_date = creation_date.context("`CreationDate` not found in attributes")?;
    let triple = id_to_str(&id, "posthistory.creation_date", &creation_date);
    formatter.format(&triple)?;

    if let Some(mut user_id) = user_id {
        user_id.insert(0, 'u');
        let triple = id_to_id(&id, "posthistory.user", &user_id);
        formatter.format(&triple)?;
    }

    if let Some(user_display_name) = user_display_name {
        let triple = id_to_str(&id, "posthistory.user_display_name", &user_display_name);
        formatter.format(&triple)?;
    }

    if let Some(comment) = comment {
        let triple = id_to_str(&id, "posthistory.comment", &comment);
        formatter.format(&triple)?;
    }

    if let Some(text) = text {
        let triple = id_to_str(&id, "posthistory.text", &text);
        formatter.format(&triple)?;
    }

    if let Some(content_license) = content_license {
        let triple = id_to_str(&id, "posthistory.content_license", &content_license);
        formatter.format(&triple)?;
    }

    Ok(())
}

fn write_postlink<W: Write>(formatter: &mut NTriplesFormatter<W>, e: &BytesStart) -> Result<()> {
    let mut id = None;
    let mut creation_date = None;
    let mut post_id = None;
    let mut related_post_id = None;
    let mut link_type = None;

    for attribute in e.attributes() {
        let attribute = attribute.context("could not parse attribute")?;
        match attribute.key {
            b"Id" => &mut id,
            b"CreationDate" => &mut creation_date,
            b"PostId" => &mut post_id,
            b"RelatedPostId" => &mut related_post_id,
            b"LinkTypeId" => &mut link_type,
            _ => continue,
        }
        .replace(parse_attribute(attribute)?);
    }

    let mut id = id.context("`Id` not found in attributes")?;
    id.insert(0, 'l');

    let creation_date = creation_date.context("`CreationDate` not found in attributes")?;
    let triple = id_to_str(&id, "postlink.creation_date", &creation_date);
    formatter.format(&triple)?;

    let mut post_id = post_id.context("`PostId` not found in attributes")?;
    post_id.insert(0, 'p');
    let triple = id_to_id(&id, "postlink.post", &post_id);
    formatter.format(&triple)?;

    let mut related_post_id = related_post_id.context("`RelatedPostId` not found in attributes")?;
    related_post_id.insert(0, 'p');
    let triple = id_to_id(&id, "postlink.related_post", &related_post_id);
    formatter.format(&triple)?;

    let link_type = link_type.context("`LinkType` not found in attributes")?;
    let triple = id_to_str(&id, "postlink.link_type", &link_type);
    formatter.format(&triple)?;

    Ok(())
}

fn write_tag<W: Write>(formatter: &mut NTriplesFormatter<W>, e: &BytesStart) -> Result<()> {
    let mut name = None;
    let mut count = None;
    let mut excerpt_post_id = None;
    let mut wiki_post_id = None;

    for attribute in e.attributes() {
        let attribute = attribute.context("could not parse attribute")?;
        match attribute.key {
            b"TagName" => &mut name,
            b"Count" => &mut count,
            b"ExcerptPostId" => &mut excerpt_post_id,
            b"WikiPostId" => &mut wiki_post_id,
            _ => continue,
        }
        .replace(parse_attribute(attribute)?);
    }

    let name = name.context("`Name` not found in attributes")?;
    let mut id = BASE32_NOPAD.encode(name.as_bytes());
    id.insert(0, 't');
    let triple = id_to_str(&id, "tag.name", &name);
    formatter.format(&triple)?;

    let count = count.context("`Count` not found in attributes")?;
    let triple = id_to_str(&id, "tag.count", &count);
    formatter.format(&triple)?;

    if let Some(mut wiki_post_id) = wiki_post_id {
        wiki_post_id.insert(0, 'p');
        let triple = id_to_id(&id, "tag.wiki_post", &wiki_post_id);
        formatter.format(&triple)?;
    }

    Ok(())
}

fn write_user<W: Write>(formatter: &mut NTriplesFormatter<W>, e: &BytesStart) -> Result<()> {
    let mut id = None;
    let mut reputation = None;
    let mut creation_date = None;
    let mut display_name = None;
    let mut last_access_date = None;
    let mut website_url = None;
    let mut location = None;
    let mut about_me = None;
    let mut views = None;
    let mut upvotes = None;
    let mut downvotes = None;
    let mut profile_image_url = None;
    let mut account_id = None;

    for attribute in e.attributes() {
        let attribute = attribute.context("could not parse attribute")?;
        match attribute.key {
            b"Id" => &mut id,
            b"Reputation" => &mut reputation,
            b"CreationDate" => &mut creation_date,
            b"DisplayName" => &mut display_name,
            b"LastAccessDate" => &mut last_access_date,
            b"WebsiteUrl" => &mut website_url,
            b"Location" => &mut location,
            b"AboutMe" => &mut about_me,
            b"Views" => &mut views,
            b"UpVotes" => &mut upvotes,
            b"DownVotes" => &mut downvotes,
            b"ProfileImageUrl" => &mut profile_image_url,
            b"AccountId" => &mut account_id,
            _ => continue,
        }
        .replace(parse_attribute(attribute)?);
    }

    let mut id = id.context("`Id` not found in attributes")?;
    id.insert(0, 'u');

    let reputation = reputation.context("`Reputation` not found in attributes")?;
    let triple = id_to_str(&id, "user.reputation", &reputation);
    formatter.format(&triple)?;

    let creation_date = creation_date.context("`CreationDate` not found in attributes")?;
    let triple = id_to_str(&id, "user.creation_date", &creation_date);
    formatter.format(&triple)?;

    let display_name = display_name.context("`Displayname` not found in attributes")?;
    let triple = id_to_str(&id, "user.display_name", &display_name);
    formatter.format(&triple)?;

    let last_access_date = last_access_date.context("`LastAccessDate` not found in attributes")?;
    let triple = id_to_str(&id, "user.last_access_date", &last_access_date);
    formatter.format(&triple)?;

    if let Some(website_url) = website_url {
        let triple = id_to_str(&id, "user.website_url", &website_url);
        formatter.format(&triple)?;
    }

    if let Some(location) = location {
        let triple = id_to_str(&id, "user.location", &location);
        formatter.format(&triple)?;
    }

    if let Some(about_me) = about_me {
        let triple = id_to_str(&id, "user.about_me", &about_me);
        formatter.format(&triple)?;
    }

    let views = views.context("`Views` not found in attributes")?;
    let triple = id_to_str(&id, "user.views", &views);
    formatter.format(&triple)?;

    let upvotes = upvotes.context("`Upvotes` not found in attributes")?;
    let triple = id_to_str(&id, "user.upvotes", &upvotes);
    formatter.format(&triple)?;

    let downvotes = downvotes.context("`Upvotes` not found in attributes")?;
    let triple = id_to_str(&id, "user.upvotes", &downvotes);
    formatter.format(&triple)?;

    if let Some(profile_image_url) = profile_image_url {
        let triple = id_to_str(&id, "user.profile_image_url", &profile_image_url);
        formatter.format(&triple)?;
    }

    if let Some(account_id) = account_id {
        let triple = id_to_str(&id, "user.account_id", &account_id);
        formatter.format(&triple)?;
    }

    Ok(())
}

fn parse_attribute(attribute: Attribute) -> Result<String> {
    let attribute = attribute
        .unescaped_value()
        .context("error escaping attribute value")?;

    String::from_utf8(attribute.into()).context("invalid utf-8 in attribute value")
}

fn id_to_str<'a>(id: &'a str, iri: &'a str, value: &'a str) -> Triple<'a> {
    Triple {
        subject: BlankNode { id }.into(),
        predicate: NamedNode { iri },
        object: Literal::Simple { value }.into(),
    }
}

fn id_to_id<'a>(id: &'a str, iri: &'a str, id_obj: &'a str) -> Triple<'a> {
    Triple {
        subject: BlankNode { id }.into(),
        predicate: NamedNode { iri },
        object: BlankNode { id: id_obj }.into(),
    }
}
