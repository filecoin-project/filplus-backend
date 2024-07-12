alter table allocators
    add column data_types text[],
    add column required_sps text,
    add column required_replicas text,
    add column registry_file_path text;