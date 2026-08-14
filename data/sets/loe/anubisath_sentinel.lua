local card = {
    api_version = 1,
    id = "LOE_061",
    name = "Anubisath Sentinel",
    text = "<b>Deathrattle:</b> Give a random friendly minion +3/+3.",
    set = "LOE",
    type = "minion",
    rarity = "common",
    cost = 5,
    attack = 4,
    health = 4,
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local candidates = ctx:friendly_minions(self)
    if #candidates > 0 then ctx:random_entity(candidates, "buff_friendly_minion") end
end

function card.buff_friendly_minion(ctx, self, target)
    ctx:buff(target, 3, 3)
end

return card
