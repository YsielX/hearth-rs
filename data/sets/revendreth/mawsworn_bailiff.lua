return {
    api_version = 1,
    id = "MAW_028",
    name = "Mawsworn Bailiff",
    text = "<b><b>Taunt</b>.</b> <b>Battlecry:</b> If you have 4 or more Armor, gain +4/+4.",
    set = "REVENDRETH",
    type = "minion",
    class = "warrior",
    cost = 5,
    attack = 4,
    health = 4,
    keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        if ctx:entity(ctx:player(player).hero).armor >= 4 then
            ctx:buff(self, 4, 4)
        end
    end,
}
