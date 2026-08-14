local card = {
    api_version = 1,
    id = "CATA_565",
    name = "Skywall Sentinel",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> <b>Herald</b> {0}.",
    set = "CATACLYSM",
    type = "minion",
    class = "shaman",
    cost = 2,
    attack = 0,
    health = 2,
    tags = { "elemental" },
    keywords = { "taunt", "battlecry", "herald" },
    keyword_params = { herald = 1 },
}

function card.on_battlecry(ctx, self) end

function card.on_herald(ctx, self)
    ctx:summon(ctx:controller(self), "CATA_565t")
end

card.tokens = {
    {
        id = "CATA_565t", name = "Soldier of Al'Akir",
        text = "[x]Adjacent minions\nhave +{0} Attack.\n<i><b>Herald</b> twice to upgrade.</i>@[x]Adjacent minions\nhave +{0} Attack.\n<i><b>Herald</b> once to upgrade.</i>@[x]Adjacent minions\nhave +{0} Attack.",
        set = "CATACLYSM", type = "minion", class = "shaman",
        cost = 1, attack = 1, health = 2, tags = { "elemental" },
        auras = {
            {
                attack = function(ctx, self)
                    local count = ctx:get_player_data(ctx:controller(self), "herald_count")
                    if count >= 4 then return 4 end
                    if count >= 2 then return 2 end
                    return 1
                end,
                targets = function(ctx, self) return ctx:adjacent_minions(self) end,
            },
        },
    },
}

return card
