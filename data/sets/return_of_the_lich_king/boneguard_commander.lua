local card = {
    api_version = 1,
    id = "RLK_506",
    name = "Boneguard Commander",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> Raise up to\n6 <b>Corpses</b> as 1/3 Risen Footmen with <b>Taunt</b>.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    rarity = "rare",
    cost = 8,
    attack = 8,
    health = 8,
    rune_cost = { blood = 1 },
    tags = { "undead" },
    keywords = { "taunt", "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local spaces = math.max(0, 7 - #ctx:board(player))
    local maximum = math.min(6, spaces)
    if maximum > 0 then
        ctx:spend_resource_and_continue(player, "corpses", 1, maximum, "raise_footmen")
    end
end

function card.raise_footmen(ctx, self, raised)
    local player = ctx:controller(self)
    for _ = 1, raised do ctx:summon(player, "RLK_061t") end
end

card.tokens = {{
    id = "RLK_061t",
    name = "Risen Footman",
    text = "<b>Taunt</b>\n<i>Doesn't leave a <b>Corpse</b>.</i>",
    set = "CORE",
    type = "minion",
    class = "death_knight",
    collectible = false,
    cost = 1,
    attack = 1,
    health = 3,
    tags = { "undead" },
    keywords = { "taunt", "no_corpse" },
}}

return card
