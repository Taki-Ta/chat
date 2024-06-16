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
