local card = {
    api_version = 1, id = "GVG_099", name = "Bomb Lobber",
    text = "<b>Battlecry:</b> Deal 4 damage to a random enemy minion.", set = "GVG",
    type = "minion", rarity = "rare", cost = 5, attack = 3, health = 3,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local targets = {}
    local enemy = ctx:opponent(ctx:controller(self))
    for _, entity in ipairs(ctx:board(enemy)) do
        if ctx:entity(entity).type == "minion" then targets[#targets + 1] = entity end
    end
    if #targets > 0 then ctx:random_entity(targets, "lob") end
end
function card.lob(ctx, self, target) cardlib.effects.damage(ctx, target, 4) end
return card
