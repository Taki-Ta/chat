--insert 3 workspace
INSERT INTO
    workspaces (name, owner_id)
VALUES
    ('ws1', '0'),
    ('ws2', '0'),
    ('ws3', '0');

--insert 5 user ,all password with hashed 'takitaki'
INSERT INTO
    users (name, email, password_hash, ws_id)
VALUES
    (
        'mitsuha',
        'mitsuha@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$uNKV4wL5Q/UrcobBhB8YWA$MRB6RydruXJnRO7/VdPkpRr4KDmRr2FlgYpnpqKoLhQ',
        0
    ),
    (
        'yotsuha',
        'yotsuha@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$uNKV4wL5Q/UrcobBhB8YWA$MRB6RydruXJnRO7/VdPkpRr4KDmRr2FlgYpnpqKoLhQ',
        0
    ),
    (
        'taki',
        'taki@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$uNKV4wL5Q/UrcobBhB8YWA$MRB6RydruXJnRO7/VdPkpRr4KDmRr2FlgYpnpqKoLhQ',
        1
    ),
    (
        'okudera',
        'okudera@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$uNKV4wL5Q/UrcobBhB8YWA$MRB6RydruXJnRO7/VdPkpRr4KDmRr2FlgYpnpqKoLhQ',
        1
    ),
    (
        'sayaka',
        'sayaka@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$uNKV4wL5Q/UrcobBhB8YWA$MRB6RydruXJnRO7/VdPkpRr4KDmRr2FlgYpnpqKoLhQ',
        0
    );

--insert 4 chats
--named chats
INSERT INTO
    chats (ws_id, name, type, members)
VALUES
    (0, 'single_chat', 'single', '{1,2}'),
    (0, 'group_chat', 'group', '{1,2,3,4,5}'),
    (0, 'private_chat', 'private_channel', '{1,2,3,4,5}'),
    (0, 'public_chat', 'public_channel', '{1,2,3,4,5}');
    --unnamed chat
INSERT INTO
    chats (ws_id, type, members)
VALUES
    (0, 'single', '{1,2}'),
    (0, 'group', '{1,2,3,4,5}');
