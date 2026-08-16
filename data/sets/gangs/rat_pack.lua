return {
    api_version = 1,
    id = "CFM_316",
    name = "Rat Pack",
    text = "[x]<b>Deathrattle:</b> Summon a\nnumber of 1/1 Rats equal\n to this minion's Attack.",
    set = "GANGS",
    type = "minion",
    class = "hunter",
    rarity = "epic",
    cost = 3,
    attack = 2,
    health = 2,
    tags = { "beast" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self, position)
        local me = ctx:entity(self)
        local count = math.max(0, me.attack_at_death or me.attack or 0)
        for _ = 1, count do ctx:summon_at(ctx:controller(self), "CFM_316t", position) end
    end,
    tokens = {{
        id = "CFM_316t", name = "Rat", text = "", set = "GANGS",
        type = "minion", class = "hunter", cost = 1, attack = 1, health = 1,
        tags = { "beast" },
    }},
}
