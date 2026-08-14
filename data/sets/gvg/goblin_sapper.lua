return {
    api_version = 1,
    id = "GVG_095",
    name = "Goblin Sapper",
    text = "Has +4 Attack while your opponent has 6 or more cards in hand.",
    set = "GVG",
    type = "minion",
    cost = 3,
    attack = 2,
    health = 4,

    auras = {
        {
            attack = function(ctx, self)
                local opponent = ctx:opponent(ctx:controller(self))
                if #ctx:hand(opponent) >= 6 then
                    return 4
                end
                return 0
            end,
            targets = function(ctx, self)
                return { self }
            end,
        },
    },
}
