-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL,
    permission_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

INSERT INTO role_permissions (role_id, permission_id)
VALUES
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', 'c5a236a3-f985-4b94-bb09-39f4c0d7f898'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', '72b27681-7537-4869-b870-28f61037e154'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', 'c3ee1a81-aad8-44a7-ab07-890e7d0a759a'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', '50d71cf1-19c3-49d3-b670-e5e0c31b079f'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', '8dd90a7a-a1b9-4ae7-be90-4051071daa1b'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', 'b277ca4b-8be4-4119-95e3-26b2089021c5'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', 'ea2ab559-3a6b-4320-8d32-41c11b092a44'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', '3f1c5226-7e68-4034-98d0-8dfa19c8d4e6'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', '463d9784-e62b-4d51-a98e-d5ecd04d0947'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', 'a3e14d41-62b5-48d8-9a1d-c95117a78b25'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', '83e2c6f1-f425-45cc-b360-a7f6eb9f57e7'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', '59dc7703-b90b-472a-8722-5785406e836b'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', '9c1c850c-f2ff-4d99-a84d-74f3fef78304'),
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', '86329004-3735-41ae-93a3-8c0fe965845f'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', 'c5a236a3-f985-4b94-bb09-39f4c0d7f898'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '72b27681-7537-4869-b870-28f61037e154'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', 'c3ee1a81-aad8-44a7-ab07-890e7d0a759a'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '0f75366c-2df2-4b74-94e8-dc4cdcc6e9ce'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '50d71cf1-19c3-49d3-b670-e5e0c31b079f'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '8dd90a7a-a1b9-4ae7-be90-4051071daa1b'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', 'b277ca4b-8be4-4119-95e3-26b2089021c5'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', 'ea2ab559-3a6b-4320-8d32-41c11b092a44'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '3f1c5226-7e68-4034-98d0-8dfa19c8d4e6'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '463d9784-e62b-4d51-a98e-d5ecd04d0947'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', 'a3e14d41-62b5-48d8-9a1d-c95117a78b25'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '83e2c6f1-f425-45cc-b360-a7f6eb9f57e7'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '59dc7703-b90b-472a-8722-5785406e836b'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', 'fd408311-636b-44fe-8f93-d847acbc3ccf'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '9c1c850c-f2ff-4d99-a84d-74f3fef78304'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', '86329004-3735-41ae-93a3-8c0fe965845f');