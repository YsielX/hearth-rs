return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_03bp",
    name = "Dagger Mastery",
    text = "<b>Hero Power</b>\nEquip a 1/2 Dagger.",
    set = "LEGACY",
    class = "rogue",
    cost = 2,
    on_play = function(ctx, self)
        ctx:equip_weapon(ctx:controller(self), "CS2_082")
    end,
    tokens = {
        {
            id = "CS2_082", name = "Wicked Knife", text = "",
            set = "LEGACY", type = "weapon", class = "rogue",
            cost = 1, attack = 1, health = 2,
        },
    },
}
