ALTER TABLE applications
    ADD COLUMN "issue_number" bigint;

UPDATE applications SET issue_number = (application::json->>'Issue Number')::bigint;

ALTER TABLE applications
    ALTER COLUMN issue_number SET NOT NULL;

CREATE UNIQUE INDEX application_owner_repo_issue_number_pr_number
ON applications USING btree
(owner, repo, issue_number, pr_number);