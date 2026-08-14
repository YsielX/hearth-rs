local KEY = "hero_power_next_turn_surcharge"
local COUNT_KEY = KEY .. ":count"
local EXPIRES_KEY = KEY .. ":expires"

local card = {
    api_version = 1, id = "AT_086", name = "Saboteur",
    text = "<b>Battlecry:</b> Your opponent's Hero Power costs (5) more next turn.",
    set = "TGT", type = "minion", rarity = "rare", cost = 3, attack = 4, health = 3,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local opponent = ctx:opponent(ctx:controller(self))
    ctx:set_player_data(opponent, COUNT_KEY, ctx:get_player_data(opponent, COUNT_KEY) + 1)
    ctx:set_player_data(opponent, EXPIRES_KEY, ctx:turn() + 1)
    ctx:grant_player_keyword(opponent, KEY)
end

return card
