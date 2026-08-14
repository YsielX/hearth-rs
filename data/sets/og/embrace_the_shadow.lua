return {
    api_version = 1,
    id = "OG_104",
    name = "Embrace the Shadow",
    text = "This turn, your healing effects deal damage instead.",
    set = "OG",
    type = "spell",
    class = "priest",
    rarity = "epic",
    spell_school = "shadow",
    cost = 2,
    on_play = function(ctx, self)
        ctx:grant_player_keyword(ctx:controller(self), "healing_becomes_damage")
    end,
}
