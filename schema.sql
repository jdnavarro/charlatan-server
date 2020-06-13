CREATE TABLE podcast (
  id INTEGER PRIMARY KEY NOT NULL,
  src TEXT UNIQUE NOT NULL,
  url TEXT NOT NULL,
  title TEXT NOT NULL,
  image TEXT NOT NULL,
  description TEXT NOT NULL
);

CREATE TABLE episode (
  id INTEGER PRIMARY KEY NOT NULL,
  guid TEXT UNIQUE NOT NULL,
  title TEXT NOT NULL,
  src TEXT NOT NULL,
  progress INTEGER NOT NULL,
  duration INTEGER NOT NULL,
  publication INTEGER NOT NULL,
  image TEXT NOT NULL,
  -- UNIQUE DEFERRABLE INITIALLY DEFERRED is not supported in sqlite
  position INTEGER,
  notes TEXT NOT NULL,
  podcast INTEGER NOT NULL,
  FOREIGN KEY (podcast) REFERENCES podcast (id)
  );
