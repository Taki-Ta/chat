-- Add migration script here

--add a function: if chat changed, notify with chat data
CREATE OR REPLACE FUNCTION add_to_chat()
RETURNS TRIGGER AS $$
BEGIN
    RAISE NOTICE 'add_to_chat: %',NEW;
    PERFORM
        pg_notify('chat_updated', json_build_object('op',TG_OP,'old',OLD,'new',NEW)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- create trigger for chat table
CREATE TRIGGER add_to_chat_trigger
AFTER INSERT OR UPDATE OR DELETE ON chats
FOR EACH ROW
EXECUTE FUNCTION add_to_chat();

-- add a function: if message changed, notify with message data
CREATE OR REPLACE FUNCTION add_to_message()
RETURNS TRIGGER AS $$
BEGIN
    RAISE NOTICE 'add_to_message: %',NEW;
    PERFORM
        pg_notify('chat_message_created', row_to_json(NEW)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- create trigger for message table
CREATE TRIGGER add_to_message_trigger
AFTER INSERT ON messages
FOR EACH ROW
EXECUTE FUNCTION add_to_message();
