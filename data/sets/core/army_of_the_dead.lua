return {
    api_version = 1,
    id = "RLK_060",
    name = "Army of the Dead",
    text = "Raise up to 5 <b>Corpses</b> as 2/2 Risen Ghouls with <b>Rush</b>.",
    set = "CORE",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    spell_school = "shadow",
    cost = 5,
    rune_cost = { unholy = 1 },
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        local spaces = math.max(0, 7 - ctx:player(player).board_size)
        local raised = ctx:spend_up_to_corpses(player, math.min(5, spaces))
        for _ = 1, raised do
            ctx:summon(player, "RLK_008t")
        end
    end,
}
