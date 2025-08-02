CREATE TABLE hits (
    -- hit id, UUID4 string
    id TEXT UNIQUE PRIMARY KEY,
    -- song title
    title TEXT NOT NULL,
    -- song artist
    artist TEXT NOT NULL,
    -- id of the YouTube video
    yt_id TEXT NOT NULL,
    -- year of release
    year INTEGER NOT NULL,
    -- offset to cut off after downloading, will start playing here
    playback_offset INTEGER NOT NULL,
    -- date of last modification
    last_modified TEXT NOT NULL,
    -- wether the hit was downloaded or not (boolean)
    downloaded BOOLEAN NOT NULL,
    -- wether the hit was imported from the codebase (false) or created manually (true) (boolean)
    custom BOOLEAN NOT NULL,
    -- wether the hit was removed (true) or not (false) (boolean)
    -- we don't just remove the hit or else it'll be restored on next launch
    marked_for_deletion BOOLEAN NOT NULL
) WITHOUT ROWID;

CREATE TABLE packs (
    -- pack id, UUID4 string
    id TEXT UNIQUE PRIMARY KEY,
    -- name of the pack
    name TEXT NOT NULL,
    -- date of last modification
    last_modified TEXT NOT NULL,
    -- wether the pack was imported from the codebase (false) or created manually (true) (boolean)
    custom BOOLEAN NOT NULL,
    -- wether the pack was removed (true) or not (false) (boolean)
    -- we don't just remove the pack or else it'll be restored on next launch
    marked_for_deletion BOOLEAN NOT NULL
) WITHOUT ROWID;

CREATE TABLE hits_packs (
    -- hit id, UUID4 string
    hit_id TEXT NOT NULL,
    -- pack id, UUID4 string
    pack_id TEXT NOT NULL,
    -- wether the association was imported from the codebase (false) or created manually (true) (boolean)
    custom BOOLEAN NOT NULL,
    -- wether the association was removed (true) or not (false) (boolean)
    -- we don't just remove the associations or else they'll be restored on next launch
    marked_for_deletion BOOLEAN NOT NULL,
    PRIMARY KEY (hit_id, pack_id),
    FOREIGN KEY (hit_id) REFERENCES hits (id) ON DELETE CASCADE,
    FOREIGN KEY (pack_id) REFERENCES packs (id) ON DELETE CASCADE
) WITHOUT ROWID;
