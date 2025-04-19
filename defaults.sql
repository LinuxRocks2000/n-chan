/* set up database defaults AFTER running init.sql (this is an optional step) */

INSERT INTO boards VALUES ('gon', 'everything n-going and n-gone', 0);
INSERT INTO boards VALUES ('cad', 'hehe fuck solidworks', 1);
INSERT INTO boards VALUES ('po', 'its like yaoi megathread but for fags', 2);
INSERT INTO boards VALUES ('yaoi', 'delicious men', 3);
INSERT INTO boards VALUES ('yuri', 'posting here will get you permabanned', 4);
INSERT INTO boards VALUES ('tttt', 'idfk', 5);

INSERT INTO users VALUES ('admin', 'changeme', 100); /* create a default admin user with full access rights and a changeme password (fucking change it you retards) */