-- Migration number: 0001 	 2025-08-17T05:39:59.584Z
CREATE TABLE images  (
      id TEXT PRIMARY KEY NOT NULL,
      created TEXT NOT NULL,
      updated TEXT NOT NULL,
      deleted TEXT,
      captured TEXT NOT NULL,
      published TEXT,
      path TEXT NOT NULL,
      caption TEXT,
      views INTEGER DEFAULT 0
);