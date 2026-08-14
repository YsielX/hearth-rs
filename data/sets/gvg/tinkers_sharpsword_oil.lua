local card = {
    api_version = 1,
    id = "GVG_022",
    name = "Tinker's Sharpsword Oil",
    text = "Give your weapon +3 Attack. <b>Combo:</b> Give a random friendly minion +3 Attack.",
    set = "GVG",
    type = "spell",
    class = "rogue",
    rarity = "common",
    cost = 4,
    keywords = { "combo" },
}

function card.on_play(ctx, self)
    local weapon = ctx:player(ctx:controller(self)).weapon
    if weapon ~= nil then ctx:buff(weapon, 3, 0) end
end

function card.on_combo(ctx, self)
    local candidates = ctx:friendly_minions(self)
    if #candidates > 0 then ctx:random_entity(candidates, "oil_minion") end
end

function card.oil_minion(ctx, self, target)
    ctx:buff(target, 3, 0)
end

return card
