return {
    api_version = 1, id = "CFM_342", name = "Luckydo Buccaneer",
    text = "<b>Battlecry:</b> If your weapon has at least 3 Attack, gain +4/+4.",
    set = "GANGS", type = "minion", class = "rogue", rarity = "epic",
    cost = 6, attack = 5, health = 5, tags = { "pirate" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local weapon = ctx:player(ctx:controller(self)).weapon
        if weapon and ctx:entity(weapon).attack >= 3 then cardlib.effects.buff(ctx, self, 4, 4) end
    end,
}
