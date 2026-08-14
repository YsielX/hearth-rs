local card = {
    api_version = 1, id = "UNG_065", name = "Sherazin, Corpse Flower",
    text = "<b>Deathrattle:</b> Go <b>Dormant</b>. Play 4 cards in a turn to revive this minion.",
    set = "UNGORO", type = "minion", class = "rogue", rarity = "legendary",
    cost = 4, attack = 6, health = 3, keywords = { "deathrattle" },
}
function card.on_deathrattle(ctx, self, position)
    ctx:transform(self, "UNG_065t")
    ctx:summon_existing_at(ctx:controller(self), self, position)
end
local seed = {
    id = "UNG_065t", name = "Sherazin, Seed",
    text = "<b>Dormant</b>\nWhen you play 4 cards in a turn, revive this minion.",
    set = "UNGORO", type = "minion", class = "rogue", cost = 11, attack = 0, health = 1,
    keywords = { "dormant" },
}
local function ready(ctx, self, player)
    return player == ctx:controller(self) and ctx:cards_played_this_turn(player) >= 4
end
seed.triggers = {
    {
        event = "minion_summoned", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.entity == self and ready(ctx, self, event.player) end,
        effect = function(ctx, self) ctx:transform(self, "UNG_065") end,
    },
    {
        event = "card_played", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return ready(ctx, self, event.player) end,
        effect = function(ctx, self) ctx:transform(self, "UNG_065") end,
    },
}
card.tokens = { seed }
return card
