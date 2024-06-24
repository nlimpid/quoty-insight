
migrate:
	sea-orm-cli generate migrate up

migrate_status:
	sea-orm-cli migrate status

generate_schema:
	sea-orm-cli generate entity -o src/entities