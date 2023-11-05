CREATE TABLE comments (
    id character(36) NOT NULL,
    group_id character varying(255) NOT NULL,
    text character varying(1024) NOT NULL,
    state int NOT NULL,
    CONSTRAINT comments_pk PRIMARY KEY (id)
)