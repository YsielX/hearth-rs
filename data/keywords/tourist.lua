-- Tourist changes deck legality only and has no in-game trigger. The deck validator
-- consumes the card's generic deck_allowances declaration.
return {
    api_version = 1, module_type = "keyword", id = "tourist", name = "Tourist",
    required_card_fields = { "deck_allowances" },
}
