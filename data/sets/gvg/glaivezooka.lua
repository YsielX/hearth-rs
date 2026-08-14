local card = {
    api_version = 1,
    id = "GVG_043",
    name = "Glaivezooka",
    text = "<b>Battlecry:</b> Give a random friendly minion +1 Attack.",
    set = "GVG",
    type = "weapon",
    class = "hunter",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 2,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = ctx:friendly_minions(self)
    if #candidates > 0 then ctx:random_entity(candidates, "buff_minion") end
end

function card.buff_minion(ctx, self, target)
    ctx:buff(target, 1, 0)
end

return card
