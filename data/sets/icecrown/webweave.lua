return {
    api_version = 1, id = "ICC_050", name = "Webweave",
    text = "Summon two 1/2 <b>Poisonous</b> Spiders.",
    set = "ICECROWN", type = "spell", class = "druid", rarity = "common",
    spell_school = "shadow", cost = 5,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:summon(player, "ICC_832t3")
        ctx:summon(player, "ICC_832t3")
    end,
}
