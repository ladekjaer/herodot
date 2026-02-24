INSERT INTO auth.users (id, username, password)
VALUES ('671bea95-1949-40c1-a0a6-8b233fdaafd5', 'rlad', '$argon2id$v=19$m=19456,t=2,p=1$1RQOgJaikWV9ipGnqSMHKw$T/TbGWAOpGTEbLB1qdk+F56/M57HrA5sAZ4/DbF+Ucw');

INSERT INTO auth.api_keys (id, name, owner_id, token)
VALUES ('05dec7f2-9aac-42a0-bbf8-794e3e80504b', 'my_token', '671bea95-1949-40c1-a0a6-8b233fdaafd5', '99ea32d6-e0dc-4b2c-9802-6eaeaf55bbac');
