local card = {
    api_version = 1,
    id = "LOOT_388",
    name = "Fungal Enchanter",
    text = "<b>Battlecry:</b> Restore #2 Health to all friendly characters.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 3,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local targets = { ctx:player(player).hero }
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        targets[#targets + 1] = minion
    end
    ctx:heal_all(targets, 2)
end

return card
