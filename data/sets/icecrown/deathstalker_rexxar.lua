local card = {
    api_version = 1,
    id = "ICC_828",
    name = "Deathstalker Rexxar",
    text = "[x]<b>Battlecry:</b> Deal 2 damage\nto all enemy minions.",
    set = "ICECROWN",
    type = "hero",
    class = "hunter",
    rarity = "legendary",
    cost = 6,
    health = 30,
    armor = 5,
    hero_power = "ICC_828p",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local targets = {}
    for _, minion in ipairs(ctx:minions()) do
        if ctx:controller(minion) ~= ctx:controller(self) then
            targets[#targets + 1] = minion
        end
    end
    cardlib.effects.damage_all(ctx, targets, 2)
end

card.tokens = {
    { id = "ICC_828t2", name = "Stubborn Gastropod", text = "<b>Taunt</b>\n  <b>Poisonous</b>", set = "UNGORO", type = "minion", rarity = "common", collectible = false, cost = 2, attack = 1, health = 2, tags = { "beast" }, keywords = { "taunt", "poisonous" } },
    { id = "ICC_828t3", name = "Giant Wasp", text = "<b>Stealth</b>\n <b>Poisonous</b>", set = "UNGORO", type = "minion", rarity = "common", collectible = false, cost = 3, attack = 2, health = 2, tags = { "beast" }, keywords = { "stealth", "poisonous" } },
    { id = "ICC_828t4", name = "Stoneskin Basilisk", text = "<b>Divine Shield</b>\n <b>Poisonous</b>", set = "LOOTAPALOOZA", type = "minion", rarity = "common", collectible = false, cost = 3, attack = 1, health = 1, tags = { "beast" }, keywords = { "divine_shield", "poisonous" } },
    { id = "ICC_828t5", name = "Hunting Mastiff", text = "<b>Echo</b>\n<b>Rush</b>", set = "GILNEAS", type = "minion", class = "hunter", rarity = "common", collectible = false, cost = 2, attack = 2, health = 1, tags = { "beast" }, keywords = { "rush" } },
    { id = "ICC_828t6", name = "Vilebrood Skitterer", text = "<b>Poisonous</b>\n<b>Rush</b>", set = "GILNEAS", type = "minion", class = "hunter", rarity = "common", collectible = false, cost = 5, attack = 1, health = 3, tags = { "beast" }, keywords = { "poisonous", "rush" } },
    { id = "ICC_828t7", name = "Vicious Scalehide", text = "<b>Lifesteal</b>\n<b>Rush</b>", set = "GILNEAS", type = "minion", rarity = "common", collectible = false, cost = 2, attack = 1, health = 3, tags = { "beast" }, keywords = { "lifesteal", "rush" } },
}

return card
