return {
    api_version = 1, id = "ICC_018", name = "Phantom Freebooter",
    text = "<b>Battlecry:</b> Gain stats equal to your weapon's.",
    set = "ICECROWN", type = "minion", rarity = "rare",
    cost = 4, attack = 3, health = 3, tags = { "undead", "pirate" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local weapon = ctx:player(ctx:controller(self)).weapon
        if weapon ~= nil then
            local snapshot = ctx:entity(weapon)
            cardlib.effects.buff(ctx, self, snapshot.attack, math.max(0, snapshot.health))
        end
    end,
}
