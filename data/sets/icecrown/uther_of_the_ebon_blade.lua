local card = {
    api_version = 1,
    id = "ICC_829",
    name = "Uther of the Ebon Blade",
    text = "<b>Battlecry:</b> Equip a 5/3 <b>Lifesteal</b> weapon.",
    set = "ICECROWN",
    type = "hero",
    class = "paladin",
    cost = 9,
    health = 30,
    armor = 5,
    hero_power = "ICC_829p",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:equip_weapon(ctx:controller(self), "ICC_829t")
end

card.tokens = {
    {
        id = "ICC_829t", name = "Grave Vengeance", text = "<b>Lifesteal</b>",
        set = "ICECROWN", type = "weapon", class = "paladin",
        cost = 8, attack = 5, health = 3, keywords = { "lifesteal" },
    },
}

return card
