local function battlecry_targets(ctx, self)
    if #ctx:secrets(ctx:controller(self)) == 0 then return {} end
    return ctx:characters()
end

local card = {
    api_version = 1,
    id = "KAR_092",
    name = "Medivh's Valet",
    text = "<b>Battlecry:</b> If you control a <b>Secret</b>, deal 3 damage.",
    set = "KARA",
    type = "minion",
    class = "mage",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 3,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = battlecry_targets,
}

function card.on_battlecry(ctx, self, target)
    if target ~= nil and #ctx:secrets(ctx:controller(self)) > 0 then
        cardlib.effects.damage(ctx, target, 3)
    end
end

return card
