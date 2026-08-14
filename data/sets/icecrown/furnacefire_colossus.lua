return {
    api_version = 1, id = "ICC_096", name = "Furnacefire Colossus",
    text = "<b>Battlecry:</b> Discard all weapons from your hand and gain their stats.",
    set = "ICECROWN", type = "minion", rarity = "epic",
    cost = 6, attack = 6, health = 6, tags = { "undead" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local attack, health, weapons = 0, 0, {}
        for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
            local snapshot = ctx:entity(entity)
            if snapshot.type == "weapon" then
                weapons[#weapons + 1] = entity
                attack = attack + snapshot.attack
                health = health + math.max(0, snapshot.health)
            end
        end
        for _, weapon in ipairs(weapons) do ctx:discard(ctx:controller(self), weapon) end
        if attack ~= 0 or health ~= 0 then ctx:buff(self, attack, health) end
    end,
}
