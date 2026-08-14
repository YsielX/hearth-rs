return {
    api_version = 1,
    id = "GVG_061",
    name = "Muster for Battle",
    text = "Summon three {0} Silver Hand Recruits. Equip a 1/4 Weapon.",
    set = "GVG",
    type = "spell",
    class = "paladin",
    rarity = "rare",
    cost = 3,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:summon(player, "CS2_101t")
        ctx:summon(player, "CS2_101t")
        ctx:summon(player, "CS2_101t")
        ctx:equip_weapon(player, "CS2_091")
    end,
}
