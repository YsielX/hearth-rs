local KEY = "mograine_end_turn_damage"

local card = {
    api_version = 1,
    id = "RLK_706",
    name = "Alexandros Mograine",
    text = "<b>Battlecry:</b> For the rest of the game, deal 3 damage to your opponent at the end of your turns.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    rarity = "legendary",
    cost = 7,
    attack = 7,
    health = 7,
    rune_cost = { blood = 3 },
    tags = { "undead" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    ctx:increment_player_data(player, KEY, 3)
    ctx:grant_public_player_keyword(player, "mograine")
end

return card
