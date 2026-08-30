return {
    api_version = 1,
    id = "CFM_305",
    name = "Smuggler's Run",
    text = "Give all minions in your hand +1/+1.",
    set = "GANGS",
    type = "spell",
    class = "paladin",
    rarity = "common",
    cost = 1,

    on_play = function(ctx, self)
        local player = ctx:controller(self)
        for _, entity in ipairs(ctx:hand(player)) do
            if ctx:entity(entity).type == "minion" then
                cardlib.effects.buff(ctx, entity, 1, 1)
            end
        end
    end,
}
