local card = {
    api_version = 1,
    id = "FP1_023",
    name = "Dark Cultist",
    text = "<b>Deathrattle:</b> Give a random friendly minion +3 Health.",
    set = "NAXX",
    type = "minion",
    class = "priest",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 4,
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local candidates = ctx:friendly_minions(self)
    if #candidates > 0 then ctx:random_entity(candidates, "give_health") end
end

function card.give_health(ctx, self, target)
    ctx:buff(target, 0, 3)
end

return card
