return {
    api_version = 1,
    id = "CS2_013",
    name = "Wild Growth",
    text = "Gain an empty Mana Crystal.",
    set = "LEGACY",
    type = "spell",
    class = "druid",
    cost = 2,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        if ctx:player(player).max_mana >= 10 then
            ctx:give_card(player, "CS2_013t")
        else
            ctx:gain_mana_crystals(player, 1, false)
        end
    end,
    tokens = {
        {
            id = "CS2_013t",
            name = "Excess Mana",
            text = "Draw a card. <i>(You can only have 10 Mana in your tray.)</i>",
            set = "LEGACY",
            type = "spell",
            class = "druid",
            cost = 0,
            on_play = function(ctx, self)
                ctx:draw(ctx:controller(self), 1)
            end,
        },
    },
}
